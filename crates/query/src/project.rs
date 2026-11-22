use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
};

use ignore::{WalkBuilder, types::TypesBuilder};
use oxc_allocator::Allocator;
use oxc_resolver::{ResolveOptions, Resolver, TsconfigDiscovery};
use petgraph::{
    algo::kosaraju_scc,
    csr::DefaultIx,
    graph::{DiGraph, NodeIndex},
    visit::{EdgeFiltered, EdgeRef, NodeFiltered},
};
use rayon::prelude::*;

use crate::{
    Export, Import,
    ImportOrExportKind::{self, Type, Value},
    Unsupported, VisitError,
    parse::{ParseResult, PendingExport, Span, parse},
};

pub struct Project {
    pub graph: Graph,
    pub errors: Vec<ModuleError>,
    pub unsupported: Vec<UnsupportedOccurrence>,
}

pub struct ModuleError {
    pub module_id: ModuleId,
    pub error: Error,
}

pub struct UnsupportedOccurrence {
    pub module_id: ModuleId,
    pub unsupported: Unsupported,
}

impl Project {
    /// Create a new project. Assumes sources are absolute paths that exist and are directories
    pub fn new<P: AsRef<Path>>(sources: Vec<P>) -> Result<Self, Error> {
        let mut graph = Graph::new();
        let sources: HashMap<PathBuf, SourceId> = sources
            .into_iter()
            .map(|p| {
                let path = p.as_ref().to_path_buf();
                let source = Source { path: path.clone() };
                let source_id = graph.insert_source(source);
                (path, source_id)
            })
            .collect();

        let mut types_builder = TypesBuilder::new();
        types_builder.add("source", "package.json")?;
        types_builder.add("source", "*.{js,cjs,mjs,ts,tsx,cts,mts}")?;
        types_builder.add("declaration", "*.d.ts")?;
        types_builder.select("source");
        types_builder.negate("declaration");

        let types = types_builder.build()?;

        let files = sources.iter().flat_map(|(source, source_id)| {
            let walker = WalkBuilder::new(source)
                .types(types.clone())
                .filter_entry(|entry| {
                    let ignored = ["node_modules", "dist", "lib", "lib-esm"];
                    let is_dir = entry.file_type().is_some_and(|ty| ty.is_dir());

                    !(is_dir && ignored.iter().any(|name| entry.file_name() == *name))
                })
                .build();

            walker.filter_map(|entry| match entry {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        Some((entry.into_path(), *source_id))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
        });

        let mut workspaces = BTreeMap::<PathBuf, WorkspaceId>::new();
        let mut modules = HashSet::<PathBuf>::new();

        for (path, source_id) in files {
            if path.file_name() == Some(std::ffi::OsStr::new("package.json")) {
                let Some(directory) = path.parent() else {
                    continue;
                };
                let directory = directory.to_path_buf();
                let workspace = Workspace {
                    path: directory.clone(),
                };
                let workspace_id = graph.insert_workspace(&source_id, workspace);
                workspaces.insert(directory, workspace_id);
            } else {
                modules.insert(path);
            }
        }

        for workspace_id in workspaces.values() {
            let workspace = graph.workspace(workspace_id);
            let package_json_path = workspace.package_json_path();
            let contents = std::fs::read_to_string(&package_json_path)?;
            let package_json: PackageJson = serde_json::from_str(&contents)?;

            if package_json.workspaces.is_empty() {
                continue;
            }

            let mut builder = globset::GlobSetBuilder::new();

            for pattern in &package_json.workspaces {
                let glob = globset::GlobBuilder::new(pattern)
                    .literal_separator(true)
                    .build()?;
                builder.add(glob);
            }

            let patterns = builder.build()?;
            let workspace_directory = workspace.path.clone();

            for (directory, child_workspace_id) in workspaces.range(workspace_directory.clone()..) {
                let Ok(relative) = directory.strip_prefix(&workspace_directory) else {
                    break;
                };

                if relative.as_os_str().is_empty() {
                    continue;
                }

                if patterns.is_match(relative) {
                    graph.insert_workspace_child(workspace_id, child_workspace_id);
                }
            }
        }

        let module_ids = modules
            .into_iter()
            .map(|path| {
                let workspace_id = path.ancestors().skip(1).find_map(|dir| workspaces.get(dir));
                let module_path = path.clone();
                let module = Module { path };
                let module_id = if let Some(workspace_id) = workspace_id {
                    graph.insert_workspace_module(workspace_id, module)
                } else {
                    graph.insert_module(module)
                };

                (module_path, module_id)
            })
            .collect::<HashMap<_, _>>();

        let parse_results = module_ids
            .par_iter()
            .map_init(Allocator::new, |allocator, (_, &module_id)| {
                let result = with_arena(allocator, |allocator| {
                    let module = graph.module(&module_id);
                    let source = std::fs::read_to_string(&module.path)?;
                    let source_type = oxc_span::SourceType::from_path(&module.path)?;
                    Ok(parse(allocator, &source, source_type)?)
                });
                (module_id, result)
            })
            .collect::<Vec<(ModuleId, Result<ParseResult, Error>)>>();

        let mut errors = Vec::new();
        let mut unsupported = Vec::new();
        let mut parsed_modules = Vec::new();

        for (module_id, result) in parse_results {
            let mut ret = match result {
                Ok(ret) => ret,
                Err(error) => {
                    errors.push(ModuleError { module_id, error });
                    continue;
                }
            };

            errors.extend(ret.errors.drain(..).map(|error| ModuleError {
                module_id,
                error: error.into(),
            }));

            unsupported.extend(ret.unsupported.drain(..).map(|unsupported| {
                UnsupportedOccurrence {
                    module_id,
                    unsupported,
                }
            }));

            parsed_modules.push((module_id, ret));
        }

        link_modules(&mut graph, &module_ids, &parsed_modules);
        Ok(Self {
            graph,
            errors,
            unsupported,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to parse package.json: {0}")]
    PackageJsonParseError(#[from] serde_json::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Failed to build types: {0}")]
    TypesBuildError(#[from] ignore::Error),

    #[error("Failed to build glob: {0}")]
    GlobBuildError(#[from] globset::Error),

    #[error("Failed to parse module: {0}")]
    ParseError(#[from] crate::parse::ParseError),

    #[error("Failed to parse source type: {0}")]
    SourceTypeError(#[from] oxc_span::UnknownExtension),

    #[error(transparent)]
    VisitError(#[from] VisitError),
}

fn with_arena<T>(arena: &mut Allocator, run: impl FnOnce(&Allocator) -> T) -> T {
    let result = run(arena);
    arena.reset();
    result
}

pub struct Graph {
    graph: DiGraph<Node, Edge, DefaultIx>,
}

impl Graph {
    fn new() -> Self {
        Self {
            graph: DiGraph::new(),
        }
    }

    // Module ------------------------------------------------------------------

    pub fn modules(&self) -> impl Iterator<Item = ModuleId> {
        self.graph.node_indices().filter_map(|node_index| {
            let Node::Module(_) = &self.graph[node_index] else {
                return None;
            };
            Some(ModuleId(node_index))
        })
    }

    pub fn module(&self, module_id: &ModuleId) -> &Module {
        let Node::Module(module) = &self.graph[module_id.0] else {
            unreachable!("A Module node must exist for any given ModuleId")
        };
        module
    }

    pub fn find_module_by_path<P: AsRef<Path>>(&self, path: P) -> Option<ModuleId> {
        let path = path.as_ref();
        self.graph.node_indices().find_map(|node_index| {
            let Node::Module(module) = &self.graph[node_index] else {
                return None;
            };
            if module.path == path {
                Some(ModuleId(node_index))
            } else {
                None
            }
        })
    }

    fn insert_module(&mut self, module: Module) -> ModuleId {
        let node = Node::Module(module);
        ModuleId(self.graph.add_node(node))
    }

    pub fn module_workspace(&self, module_id: &ModuleId) -> Option<WorkspaceId> {
        self.graph
            .neighbors_directed(module_id.0, petgraph::Direction::Incoming)
            .find_map(|workspace_index| {
                let Node::Workspace(_) = &self.graph[workspace_index] else {
                    return None;
                };
                Some(WorkspaceId(workspace_index))
            })
    }

    /// Indexed modules referenced by this module's imports and re-exports.
    pub fn module_dependencies(&self, module_id: &ModuleId) -> impl Iterator<Item = ModuleId> {
        self.graph
            .edges(module_id.0)
            .filter(|edge| matches!(edge.weight(), Edge::ModuleDependency { .. }))
            .map(|edge| ModuleId(edge.target()))
    }

    // Module -[ModuleExport]-> ModuleExport -----------------------------------

    pub fn module_exports(&self, module_id: &ModuleId) -> impl Iterator<Item = ModuleExportId> {
        self.graph
            .edges(module_id.0)
            .filter(|edge| matches!(edge.weight(), Edge::ModuleExport))
            .map(|edge| ModuleExportId(edge.target()))
    }

    pub fn module_export(&self, module_export_id: &ModuleExportId) -> &ModuleExport {
        let Node::ModuleExport(module_export) = &self.graph[module_export_id.0] else {
            unreachable!("A ModuleExport node must exist for any given ModuleExportId")
        };
        module_export
    }

    fn insert_module_export(
        &mut self,
        module_id: &ModuleId,
        module_export: ModuleExport,
    ) -> ModuleExportId {
        let node = Node::ModuleExport(module_export);
        let module_export_id = ModuleExportId(self.graph.add_node(node));

        self.graph
            .add_edge(module_id.0, module_export_id.0, Edge::ModuleExport);

        module_export_id
    }

    // Source ------------------------------------------------------------------

    pub fn sources(&self) -> impl Iterator<Item = SourceId> {
        self.graph.node_indices().filter_map(|node_index| {
            let Node::Source(_) = &self.graph[node_index] else {
                return None;
            };
            Some(SourceId(node_index))
        })
    }

    pub fn source(&self, source_id: &SourceId) -> &Source {
        let Node::Source(source) = &self.graph[source_id.0] else {
            unreachable!("A Source node must exist for any given SourceId")
        };
        source
    }

    fn insert_source(&mut self, source: Source) -> SourceId {
        let node = Node::Source(source);
        SourceId(self.graph.add_node(node))
    }

    pub fn insert_workspace(&mut self, source_id: &SourceId, workspace: Workspace) -> WorkspaceId {
        let node = Node::Workspace(workspace);
        let workspace_id = WorkspaceId(self.graph.add_node(node));

        self.graph
            .add_edge(source_id.0, workspace_id.0, Edge::Source);

        workspace_id
    }

    // Workspace ---------------------------------------------------------------

    pub fn workspaces(&self) -> impl Iterator<Item = WorkspaceId> {
        self.graph.node_indices().filter_map(|node_index| {
            let Node::Workspace(_) = &self.graph[node_index] else {
                return None;
            };
            Some(WorkspaceId(node_index))
        })
    }

    pub fn workspace(&self, workspace_id: &WorkspaceId) -> &Workspace {
        let Node::Workspace(workspace) = &self.graph[workspace_id.0] else {
            unreachable!("A Workspace node must exist for any given WorkspaceId")
        };
        workspace
    }

    pub fn find_workspace_by_path<P: AsRef<Path>>(&self, directory: P) -> Option<WorkspaceId> {
        let directory = directory.as_ref();
        self.graph.node_indices().find_map(|node_index| {
            let Node::Workspace(workspace) = &self.graph[node_index] else {
                return None;
            };
            if workspace.path == directory {
                Some(WorkspaceId(node_index))
            } else {
                None
            }
        })
    }

    // Workspace -[Module]-> Module --------------------------------------------

    pub fn workspace_modules(&self, workspace_id: &WorkspaceId) -> impl Iterator<Item = ModuleId> {
        self.graph
            .neighbors_directed(workspace_id.0, petgraph::Direction::Outgoing)
            .filter_map(|module_index| {
                let Node::Module(_) = &self.graph[module_index] else {
                    return None;
                };
                Some(ModuleId(module_index))
            })
    }

    fn insert_workspace_module(&mut self, workspace_id: &WorkspaceId, module: Module) -> ModuleId {
        let module_id = self.insert_module(module);

        self.graph
            .add_edge(workspace_id.0, module_id.0, Edge::Module);

        module_id
    }

    // Workspace -[Source]-> Source --------------------------------------------

    pub fn workspace_source(&self, workspace_id: &WorkspaceId) -> SourceId {
        self.graph
            .neighbors_directed(workspace_id.0, petgraph::Direction::Incoming)
            .find_map(|source_index| {
                let Node::Source(_) = &self.graph[source_index] else {
                    return None;
                };
                Some(SourceId(source_index))
            })
            .expect("A Source node must exist for any given WorkspaceId")
    }

    // Workspace -[ChildWorkspace]-> Workspace ---------------------------------

    pub fn workspace_parent(&self, workspace_id: &WorkspaceId) -> Option<WorkspaceId> {
        self.graph
            .neighbors_directed(workspace_id.0, petgraph::Direction::Incoming)
            .find_map(|parent_index| {
                let Node::Workspace(_) = &self.graph[parent_index] else {
                    return None;
                };
                Some(WorkspaceId(parent_index))
            })
    }

    pub fn workspace_children(
        &self,
        workspace_id: &WorkspaceId,
    ) -> impl Iterator<Item = WorkspaceId> {
        self.graph
            .neighbors_directed(workspace_id.0, petgraph::Direction::Outgoing)
            .filter_map(|child_index| {
                let Node::Workspace(_) = &self.graph[child_index] else {
                    return None;
                };
                Some(WorkspaceId(child_index))
            })
    }

    fn insert_workspace_child(&mut self, parent: &WorkspaceId, child: &WorkspaceId) {
        self.graph.add_edge(parent.0, child.0, Edge::ChildWorkspace);
    }
}

enum Node {
    Module(Module),
    ModuleExport(ModuleExport),
    Source(Source),
    Workspace(Workspace),
}

enum Edge {
    ImportBinding,
    ModuleDependency {
        re_export: bool,
    },
    /// A directed edge from a Workspace to one of its child Workspaces
    ChildWorkspace,
    /// A directed edge from a Module to one of its Exports
    ModuleExport,
    /// A directed edge from a Workspace to one of its Modules
    Module,
    /// A directed edge from a Workspace to the Source that contains it
    Source,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(NodeIndex);

pub struct Source {
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorkspaceId(NodeIndex);

pub struct Workspace {
    pub path: PathBuf,
}

impl Workspace {
    fn package_json_path(&self) -> PathBuf {
        self.path.join("package.json")
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PackageJson {
    #[serde(default)]
    workspaces: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleId(NodeIndex);

pub struct Module {
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleExportId(NodeIndex);

pub struct ModuleExport {
    pub binding: String,
    pub export_kind: ImportOrExportKind,
    pub span: Span,
}

type Key<'a> = (&'a str, ImportOrExportKind);
type Exports<'a> = HashMap<Key<'a>, Resolution>;

#[derive(Clone, Copy)]
enum Target<'a> {
    Local(NodeIndex),
    Namespace(Option<ModuleId>),
    Name(Option<ModuleId>, &'a str),
}

#[derive(Default)]
struct ModuleLinks<'a> {
    imports: HashMap<&'a str, (Target<'a>, ImportOrExportKind)>,
    explicit: HashMap<Key<'a>, (Target<'a>, Span)>,
    stars: Vec<(ModuleId, ImportOrExportKind, Span)>,
}

impl<'a> ModuleLinks<'a> {
    fn alias(&mut self, key: Key<'a>, target: Target<'a>, span: Span) {
        self.explicit.insert(key, (target, span));
        if key.1 == Value && matches!(target, Target::Name(..)) {
            self.explicit.entry((key.0, Type)).or_insert((target, span));
        }
    }
}

#[derive(Clone, Copy)]
enum Resolution {
    Binding(NodeIndex, Span),
    Ambiguous,
}

fn link_modules(
    graph: &mut Graph,
    module_ids: &HashMap<PathBuf, ModuleId>,
    parsed: &[(ModuleId, ParseResult)],
) {
    let strings = |values: &[&str]| values.iter().map(|value| (*value).to_owned()).collect();
    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigDiscovery::Auto),
        condition_names: strings(&["node", "import"]),
        extensions: strings(&[".ts", ".tsx", ".mts", ".cts", ".js", ".mjs", ".cjs"]),
        extension_alias: vec![
            (".js".into(), strings(&[".js", ".ts", ".tsx"])),
            (".mjs".into(), strings(&[".mjs", ".mts"])),
            (".cjs".into(), strings(&[".cjs", ".cts"])),
        ],
        ..ResolveOptions::default()
    });
    let mut directories = HashMap::new();
    let canonical_ids = module_ids
        .iter()
        .filter_map(|(path, &id)| {
            let parent = path.parent()?;
            let canonical = directories
                .entry(parent)
                .or_insert_with(|| parent.canonicalize().ok().filter(|path| path != parent));
            Some((canonical.as_ref()?.join(path.file_name()?), id))
        })
        .collect::<HashMap<_, _>>();

    let modules = parsed
        .iter()
        .map(|(id, parsed)| {
            let mut cache = HashMap::new();
            let links = link_module(graph, *id, parsed, |path, source| {
                *cache.entry(source).or_insert_with(|| {
                    let resolved = resolver.resolve_file(path, source).ok()?;
                    module_ids
                        .get(resolved.path())
                        .or_else(|| canonical_ids.get(resolved.path()))
                        .copied()
                })
            });
            (*id, links)
        })
        .collect::<HashMap<_, _>>();

    let dependencies = EdgeFiltered::from_fn(&graph.graph, |edge| {
        matches!(edge.weight(), Edge::ModuleDependency { re_export: true })
    });
    let module_graph = NodeFiltered::from_fn(&dependencies, |id| {
        matches!(graph.graph[id], Node::Module(_))
    });
    let mut settled = HashMap::<ModuleId, Exports<'_>>::new();
    for component in kosaraju_scc(&module_graph) {
        // Only reuse complete results from earlier components, not cycle cutoffs.
        let resolved = component
            .into_iter()
            .map(ModuleId)
            .filter(|id| modules.contains_key(id))
            .map(|id| {
                let exports = [Type, Value]
                    .into_iter()
                    .flat_map(|kind| candidate_names(&modules, &settled, id, kind))
                    .filter_map(|key| Some((key, resolve(&modules, &settled, id, key)?)))
                    .collect();
                (id, exports)
            })
            .collect::<Vec<_>>();
        settled.extend(resolved);
    }

    for (id, links) in modules {
        for (&(name, kind), &result) in &settled[&id] {
            if let Resolution::Binding(_, span) = result
                && !matches!(
                    links.explicit.get(&(name, kind)),
                    Some((Target::Local(_), _))
                )
            {
                insert_export(graph, id, (name, kind), span);
            }
        }
        for (target, kind) in links.imports.into_values() {
            let target = match target {
                Target::Name(Some(provider), name) => settled.get(&provider).and_then(|exports| {
                    let result = exports.get(&(name, kind)).or_else(|| {
                        if kind == Value {
                            exports.get(&(name, Type))
                        } else {
                            None
                        }
                    });
                    match result {
                        Some(Resolution::Binding(origin, _)) => Some(*origin),
                        _ => None,
                    }
                }),
                Target::Namespace(provider) => provider.map(|id| id.0),
                _ => None,
            };
            if let Some(target) = target {
                graph.graph.add_edge(id.0, target, Edge::ImportBinding);
            }
        }
    }
}

fn link_module<'a>(
    graph: &mut Graph,
    id: ModuleId,
    parsed: &'a ParseResult,
    mut resolve: impl FnMut(&Path, &'a str) -> Option<ModuleId>,
) -> ModuleLinks<'a> {
    let mut links = ModuleLinks::default();
    let mut dependencies = HashMap::new();
    let path = &graph.module(&id).path;
    let mut dependency = |source, re_export| {
        let target = resolve(path, source)?;
        *dependencies.entry(target).or_insert(false) |= re_export;
        Some(target)
    };
    for import in &parsed.imports {
        let (local, source, name, kind) = match import {
            Import::Named(i) => (
                &i.local,
                &i.source,
                Some(i.imported.as_str()),
                i.import_kind,
            ),
            Import::Default(i) => (&i.local, &i.source, Some("default"), i.import_kind),
            Import::Namespace(i) => (&i.local, &i.source, None, i.import_kind),
        };
        let provider = dependency(source, false);
        let target = match name {
            Some(name) => Target::Name(provider, name),
            None => Target::Namespace(provider),
        };
        links.imports.insert(local, (target, kind));
    }
    for source in &parsed.side_effect_dependencies {
        dependency(source, false);
    }
    for export in &parsed.pending_exports {
        match export {
            PendingExport::Named(e) => links.alias(
                (&e.exported, e.export_kind),
                Target::Name(dependency(&e.source, true), &e.local),
                e.span,
            ),
            PendingExport::Namespace(e) => {
                let target = Target::Namespace(dependency(&e.source, true));
                links
                    .explicit
                    .insert((&e.exported, e.export_kind), (target, e.span));
            }
            PendingExport::All(e) => {
                if let Some(target) = dependency(&e.source, true) {
                    links.stars.push((target, e.export_kind, e.span));
                }
            }
        }
    }

    let mut locals = Vec::new();
    for export in parsed.exports.values() {
        let (key, local, span) = match export {
            Export::Default(e) => (("default", Value), "default", e.span),
            Export::Named(e) => {
                let name = e.binding.as_str();
                (
                    (name, e.export_kind),
                    e.local.as_deref().unwrap_or(name),
                    e.span,
                )
            }
        };
        if let Some(&(target, kind)) = links.imports.get(local) {
            if let Target::Name(Some(module), _) | Target::Namespace(Some(module)) = target {
                dependencies.insert(module, true);
            }
            links.alias(
                (key.0, if kind == Type { Type } else { key.1 }),
                target,
                span,
            );
        } else {
            locals.push((key, local, span));
        }
    }
    for (target, re_export) in dependencies {
        graph
            .graph
            .add_edge(id.0, target.0, Edge::ModuleDependency { re_export });
    }

    // Prefer the original spelling when several exports refer to the same local.
    locals.sort_unstable_by_key(|(key, local, span)| {
        (*local != key.0, span.start.line, span.start.column)
    });
    let mut originals = HashMap::new();
    for (key, local, span) in locals {
        let node = insert_export(graph, id, key, span);
        let origin = *originals.entry((local, key.1)).or_insert(node);
        links.explicit.insert(key, (Target::Local(origin), span));
    }
    links
}

fn candidate_names<'a>(
    modules: &HashMap<ModuleId, ModuleLinks<'a>>,
    settled: &HashMap<ModuleId, Exports<'a>>,
    root: ModuleId,
    kind: ImportOrExportKind,
) -> HashSet<Key<'a>> {
    let mut names = HashSet::new();
    let mut seen = HashSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        let include =
            |&&(name, space): &&Key<'_>| space == kind && (id == root || name != "default");
        if let Some(exports) = settled.get(&id) {
            names.extend(exports.keys().filter(include).copied());
        } else if let Some(links) = modules.get(&id) {
            names.extend(links.explicit.keys().filter(include).copied());
            pending.extend(star_targets(links, kind).map(|(target, _)| target));
        }
    }
    names
}

fn resolve<'a>(
    modules: &HashMap<ModuleId, ModuleLinks<'a>>,
    settled: &HashMap<ModuleId, Exports<'a>>,
    root: ModuleId,
    key: Key<'a>,
) -> Option<Resolution> {
    if let Some(&(Target::Local(origin), span)) = modules[&root].explicit.get(&key) {
        return Some(Resolution::Binding(origin, span));
    }
    let mut seen = HashSet::new();
    let mut pending = vec![(root, key.0, None)];
    let mut binding = None;
    while let Some((id, name, span)) = pending.pop() {
        if !seen.insert((id, name)) {
            continue;
        }
        let kind = key.1;
        let result = if let Some(exports) = settled.get(&id) {
            match exports.get(&(name, kind)) {
                Some(Resolution::Binding(origin, site)) => Some((*origin, span.unwrap_or(*site))),
                Some(Resolution::Ambiguous) => return Some(Resolution::Ambiguous),
                None => None,
            }
        } else if let Some(links) = modules.get(&id) {
            if let Some(&(target, site)) = links.explicit.get(&(name, kind)) {
                let span = span.unwrap_or(site);
                match target {
                    Target::Local(origin) => Some((origin, span)),
                    Target::Namespace(module) => module.map(|id| (id.0, span)),
                    Target::Name(Some(module), imported) => {
                        pending.push((module, imported, Some(span)));
                        None
                    }
                    Target::Name(None, _) => None,
                }
            } else {
                if name != "default" {
                    pending.extend(
                        star_targets(links, kind)
                            .rev()
                            .map(|(target, site)| (target, name, span.or(Some(site)))),
                    );
                }
                None
            }
        } else {
            None
        };
        if let Some((origin, span)) = result {
            if let Some((previous, _)) = binding {
                if previous != origin {
                    return Some(Resolution::Ambiguous);
                }
            } else {
                binding = Some((origin, span));
            }
        }
    }
    binding.map(|(origin, span)| Resolution::Binding(origin, span))
}

fn star_targets(
    links: &ModuleLinks<'_>,
    kind: ImportOrExportKind,
) -> impl DoubleEndedIterator<Item = (ModuleId, Span)> {
    links
        .stars
        .iter()
        .filter_map(move |&(target, star_kind, span)| {
            (star_kind == Value || kind == Type).then_some((target, span))
        })
}

fn insert_export(graph: &mut Graph, id: ModuleId, key: Key<'_>, span: Span) -> NodeIndex {
    graph
        .insert_module_export(
            &id,
            ModuleExport {
                binding: key.0.to_owned(),
                export_kind: key.1,
                span,
            },
        )
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    fn module_id(project: &Project, directory: &Path, file: &str) -> ModuleId {
        project
            .graph
            .find_module_by_path(directory.join(file))
            .unwrap()
    }

    fn exported(graph: &Graph, module: ModuleId, name: &str) -> Option<ModuleExportId> {
        graph
            .module_exports(&module)
            .find(|id| graph.module_export(id).binding == name)
    }

    fn import_targets(graph: &Graph, module: ModuleId) -> HashSet<NodeIndex> {
        graph
            .graph
            .edges(module.0)
            .filter(|edge| matches!(edge.weight(), Edge::ImportBinding))
            .map(|edge| edge.target())
            .collect()
    }

    #[test]
    fn test_links_module_dependencies_and_bindings() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let root = directory.path();
        for (file, source) in [
            (
                "tsconfig.json",
                r#"{"compilerOptions":{"paths":{"@barrel":["./barrel.ts"]}}}"#,
            ),
            (
                "leaf.ts",
                r#"import { start } from "./entry";
                   export const x = 1;
                   export default 2;
                   export type Shape = string;"#,
            ),
            (
                "barrel.ts",
                r#"import { x as local } from "./leaf.js";
                   export { local as alias };
                   export { default as renamed } from "./leaf.js";
                   export * as ns from "./leaf.js";
                   export * from "./leaf.js";"#,
            ),
            ("side.ts", ""),
            (
                "entry.ts",
                r#"import { alias, renamed, ns, type Shape } from "@barrel";
                   import defaultFromStar from "@barrel";
                   import * as barrel from "@barrel";
                   import { missing } from "./missing";
                   import "./side";
                   export const start = 1;"#,
            ),
        ] {
            std::fs::write(root.join(file), source)?;
        }

        let project = Project::new(vec![root])?;
        assert!(project.errors.is_empty());
        let graph = &project.graph;
        let leaf = module_id(&project, root, "leaf.ts");
        let barrel = module_id(&project, root, "barrel.ts");
        let entry = module_id(&project, root, "entry.ts");
        let side = module_id(&project, root, "side.ts");
        assert_eq!(
            graph.module_dependencies(&barrel).collect::<Vec<_>>(),
            [leaf]
        );
        assert_eq!(
            graph.module_dependencies(&entry).collect::<HashSet<_>>(),
            HashSet::from([barrel, side]),
        );

        for name in ["alias", "renamed", "ns", "Shape"] {
            assert!(exported(graph, barrel, name).is_some());
        }
        assert_eq!(exported(graph, barrel, "default"), None);
        assert_eq!(exported(graph, leaf, "start"), None);
        assert_eq!(
            import_targets(graph, entry),
            HashSet::from([
                exported(graph, leaf, "x").unwrap().0,
                exported(graph, leaf, "default").unwrap().0,
                exported(graph, leaf, "Shape").unwrap().0,
                leaf.0,
                barrel.0,
            ]),
        );
        assert_eq!(
            import_targets(graph, leaf),
            HashSet::from([exported(graph, entry, "start").unwrap().0]),
        );
        Ok(())
    }

    #[test]
    fn test_resolves_export_cycles_and_star_ambiguity() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let root = directory.path();
        for (file, source) in [
            ("leaf.ts", "export const x = 1, y = 2, z = 3;"),
            (
                "a.ts",
                r#"export * from "./b";
                   export * from "./leaf";
                   export { loop } from "./b";"#,
            ),
            (
                "b.ts",
                r#"export * from "./a";
                   export { x, y } from "./leaf";
                   export { loop } from "./a";"#,
            ),
            ("other.ts", "export const y = 2, z = 3;"),
            (
                "entry.ts",
                r#"export * from "./a";
                   export * from "./b";
                   export * from "./other";
                   export { z } from "./other";"#,
            ),
            ("use.ts", r#"import { x, y, z, loop } from "./entry";"#),
        ] {
            std::fs::write(root.join(file), source)?;
        }

        let project = Project::new(vec![root])?;
        assert!(project.errors.is_empty());
        let graph = &project.graph;
        let leaf = module_id(&project, root, "leaf.ts");
        let a = module_id(&project, root, "a.ts");
        let b = module_id(&project, root, "b.ts");
        let other = module_id(&project, root, "other.ts");
        let entry = module_id(&project, root, "entry.ts");
        let consumer = module_id(&project, root, "use.ts");

        for module in [a, b, entry] {
            assert!(exported(graph, module, "x").is_some());
            assert_eq!(exported(graph, module, "loop"), None);
        }
        assert_eq!(exported(graph, entry, "y"), None);
        assert_eq!(
            import_targets(graph, consumer),
            HashSet::from([
                exported(graph, leaf, "x").unwrap().0,
                exported(graph, other, "z").unwrap().0,
            ]),
        );
        Ok(())
    }

    #[test]
    fn test_module_diagnostics() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path();
        let invalid = directory.join("invalid.ts");
        let partial = directory.join("partial.ts");

        std::fs::write(&invalid, "export const =")?;
        std::fs::write(
            &partial,
            concat!(
                "const first = 1, second = 2;\n",
                "export { first }; export { second as first };\n",
                "export const { x } = { x: 1 };\n",
                "export const [y] = [2];\n",
                "export const { z } = { z: 3 };\n",
            ),
        )?;

        let project = Project::new(vec![directory])?;
        let invalid_id = project.graph.find_module_by_path(invalid).unwrap();
        let partial_id = project.graph.find_module_by_path(partial).unwrap();
        assert_eq!(project.errors.len(), 2);
        assert!(project.errors.iter().any(|entry| {
            entry.module_id == invalid_id && matches!(entry.error, Error::ParseError(_))
        }));
        assert!(project.errors.iter().any(|entry| {
            entry.module_id == partial_id
                && matches!(
                    entry.error,
                    Error::VisitError(VisitError::DuplicateExport(_))
                )
        }));
        let occurrences = project
            .unsupported
            .iter()
            .map(|entry| {
                assert_eq!(entry.module_id, partial_id);
                (
                    entry.unsupported.ast_kind.as_str(),
                    entry.unsupported.span.start.line,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            occurrences,
            [
                ("ObjectPattern", 3),
                ("ArrayPattern", 4),
                ("ObjectPattern", 5)
            ]
        );
        assert!(project.graph.module_exports(&partial_id).next().is_some());

        Ok(())
    }

    #[test]
    fn test_single_workspace() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();

        std::fs::write(directory.join("package.json"), r#"{}"#)?;

        let project = Project::new(vec![&directory])?;

        assert!(
            project
                .graph
                .workspaces()
                .any(|id| project.graph.workspace(&id).path == directory)
        );

        Ok(())
    }

    #[test]
    fn test_multi_workspace() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();

        std::fs::write(directory.join("package.json"), r#"{}"#)?;

        let test_package_directory = directory.join("packages").join("test");
        std::fs::create_dir_all(&test_package_directory)?;
        std::fs::write(test_package_directory.join("package.json"), r#"{}"#)?;

        let project = Project::new(vec![&directory])?;

        assert!(
            project
                .graph
                .workspaces()
                .any(|id| project.graph.workspace(&id).path == directory)
        );
        assert!(
            project
                .graph
                .workspaces()
                .any(|id| project.graph.workspace(&id).path == test_package_directory)
        );

        Ok(())
    }

    #[test]
    fn test_multi_workspace_with_workspaces() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();

        std::fs::write(
            directory.join("package.json"),
            r#"{"workspaces": ["packages/*"]}"#,
        )?;

        let test_package_directory = directory.join("packages").join("test");
        std::fs::create_dir_all(&test_package_directory)?;
        std::fs::write(test_package_directory.join("package.json"), r#"{}"#)?;

        let project = Project::new(vec![&directory])?;
        let root_workspace_id = project
            .graph
            .find_workspace_by_path(&directory)
            .expect("Root workspace should exist");
        let test_workspace_id = project
            .graph
            .find_workspace_by_path(&test_package_directory)
            .expect("Test workspace should exist");

        assert!(
            project
                .graph
                .workspace_children(&root_workspace_id)
                .any(|ws_id| ws_id == test_workspace_id)
        );

        assert!(
            project
                .graph
                .workspace_parent(&test_workspace_id)
                .map(|ws_id| ws_id == root_workspace_id)
                .unwrap_or(false)
        );

        Ok(())
    }

    #[test]
    fn test_modules() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();
        let module_a_path = directory.join("a.js");
        let module_b_path = directory.join("src").join("b.ts");
        let module_c_path = directory.join("src").join("nested").join("c.tsx");

        std::fs::create_dir_all(module_c_path.parent().expect("Parent to be defined"))?;
        std::fs::write(directory.join("package.json"), "{}")?;
        std::fs::write(&module_a_path, "")?;
        std::fs::write(&module_b_path, "")?;
        std::fs::write(&module_c_path, "")?;

        let project = Project::new(vec![&directory])?;
        let workspace_id = project
            .graph
            .find_workspace_by_path(&directory)
            .expect("Workspace should exist");
        let module_a_id = project
            .graph
            .find_module_by_path(&module_a_path)
            .expect("Module A should exist");
        let module_b_id = project
            .graph
            .find_module_by_path(&module_b_path)
            .expect("Module B should exist");
        let module_c_id = project
            .graph
            .find_module_by_path(&module_c_path)
            .expect("Module C should exist");
        let workspace_module_ids = project
            .graph
            .workspace_modules(&workspace_id)
            .collect::<Vec<_>>();
        let module_ids = [module_a_id, module_b_id, module_c_id];

        assert!(
            module_ids
                .iter()
                .all(|module_id| workspace_module_ids.contains(module_id))
        );

        Ok(())
    }

    #[test]
    fn test_module_exports_empty() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();
        let module_a_path = directory.join("a.js");

        std::fs::write(directory.join("package.json"), "{}")?;
        std::fs::write(&module_a_path, "")?;

        let project = Project::new(vec![&directory])?;
        let module_a_id = project
            .graph
            .find_module_by_path(&module_a_path)
            .expect("Module A should exist");
        let module_export_ids = project
            .graph
            .module_exports(&module_a_id)
            .collect::<Vec<_>>();

        assert!(module_export_ids.is_empty());

        Ok(())
    }

    #[test]
    fn test_module_exports() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();
        let module_a_path = directory.join("a.js");

        std::fs::write(directory.join("package.json"), "{}")?;
        std::fs::write(
            &module_a_path,
            "export const foo = 42; export default 'bar';",
        )?;

        let project = Project::new(vec![&directory])?;
        let module_a_id = project
            .graph
            .find_module_by_path(&module_a_path)
            .expect("Module A should exist");
        let module_export_ids = project
            .graph
            .module_exports(&module_a_id)
            .collect::<Vec<_>>();

        assert_eq!(module_export_ids.len(), 2);
        assert!(module_export_ids.iter().any(|module_export_id| {
            let module_export = project.graph.module_export(module_export_id);
            module_export.binding == "foo" && module_export.export_kind == ImportOrExportKind::Value
        }));
        assert!(module_export_ids.iter().any(|module_export_id| {
            let module_export = project.graph.module_export(module_export_id);
            module_export.binding == "default"
                && module_export.export_kind == ImportOrExportKind::Value
        }));

        Ok(())
    }
}
