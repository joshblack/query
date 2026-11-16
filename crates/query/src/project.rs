use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
};

use ignore::{WalkBuilder, types::TypesBuilder};
use petgraph::{
    csr::DefaultIx,
    graph::{DiGraph, NodeIndex},
};

pub struct Project {
    pub graph: Graph,
    pub sources: HashSet<PathBuf>,
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
        // types_builder.add("source", "*.{js,cjs,mjs,ts,tsx,cts,mts}")?;
        types_builder.add("declaration", "*.d.ts")?;
        types_builder.select("source");
        types_builder.negate("declaration");

        let types = types_builder.build()?;

        let files = sources.iter().flat_map(|(source, source_id)| {
            let walker = WalkBuilder::new(source)
                .types(types.clone())
                .filter_entry(|entry| {
                    !(entry.file_name() == "node_modules"
                        && entry.file_type().is_some_and(|ty| ty.is_dir()))
                })
                .build();

            walker.filter_map(|entry| match entry {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        Some((entry.into_path(), *source_id))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
        });

        let workspaces = files
            .filter_map(|file: (PathBuf, SourceId)| {
                let directory = file.0.parent()?.to_path_buf();
                let workspace = Workspace {
                    path: directory.clone(),
                };
                let workspace_id = graph.insert_workspace(&file.1, workspace);
                Some((directory, workspace_id))
            })
            .collect::<BTreeMap<PathBuf, WorkspaceId>>();

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

        Ok(Self {
            graph,
            sources: sources.into_keys().collect::<HashSet<_>>(),
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

    pub fn find_workspace_by_directory<P: AsRef<Path>>(&self, directory: P) -> Option<WorkspaceId> {
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

    // Workspace -> Workspace --------------------------------------------------

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

    // Workspace -> Source -----------------------------------------------------

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
}

enum Node {
    // Module(Module),
    Source(Source),
    Workspace(Workspace),
}

enum Edge {
    ChildWorkspace,
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

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub struct ModuleId(NodeIndex);
//
// pub struct Module {
//     pub path: PathBuf,
// }

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    #[test]
    fn test_single_workspace() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();

        std::fs::write(&directory.join("package.json"), r#"{}"#)?;

        let project = Project::new(vec![&directory])?;

        assert!(project.graph.workspaces().any(|ws| ws.path == directory));

        Ok(())
    }

    #[test]
    fn test_multi_workspace() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();

        std::fs::write(&directory.join("package.json"), r#"{}"#)?;

        let test_package_directory = directory.join("packages").join("test");
        std::fs::create_dir_all(&test_package_directory)?;
        std::fs::write(&test_package_directory.join("package.json"), r#"{}"#)?;

        let project = Project::new(vec![&directory])?;

        assert!(project.graph.workspaces().any(|ws| ws.path == directory));
        assert!(
            project
                .graph
                .workspaces()
                .any(|ws| ws.path == test_package_directory)
        );

        Ok(())
    }

    #[test]
    fn test_multi_workspace_with_workspaces() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let directory = tempdir.path().to_path_buf();

        std::fs::write(
            &directory.join("package.json"),
            r#"{"workspaces": ["packages/*"]}"#,
        )?;

        let test_package_directory = directory.join("packages").join("test");
        std::fs::create_dir_all(&test_package_directory)?;
        std::fs::write(&test_package_directory.join("package.json"), r#"{}"#)?;

        let project = Project::new(vec![&directory])?;
        let root_workspace_id = project
            .graph
            .find_workspace_by_directory(&directory)
            .expect("Root workspace should exist");
        let test_workspace_id = project
            .graph
            .find_workspace_by_directory(&test_package_directory)
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
}
