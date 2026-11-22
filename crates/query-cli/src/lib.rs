use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cli_table::{Cell, Table, print_stdout};
use query::{ImportOrExportKind, list_exports, project::WorkspaceId};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Module(ModuleArgs),
    Parse(ParseArgs),
    Workspace(WorkspaceArgs),
}

#[derive(clap::Args)]
struct ModuleArgs {
    #[command(subcommand)]
    command: ModuleCommand,
}

#[derive(Subcommand)]
enum ModuleCommand {
    Export {
        #[command(subcommand)]
        command: ModuleExportCommand,
    },
}

#[derive(Subcommand)]
enum ModuleExportCommand {
    /// List the exports of a module in the project
    List {
        /// The path to the module
        path: PathBuf,
    },
}

#[derive(clap::Args)]
struct ParseArgs {
    #[command(subcommand)]
    command: ParseCommand,
}

#[derive(Subcommand)]
enum ParseCommand {
    Exports {
        #[command(subcommand)]
        action: ExportsAction,
    },
    Imports {
        #[command(subcommand)]
        action: ImportsAction,
    },
}

#[derive(Subcommand)]
enum ExportsAction {
    List {
        /// The path to the file
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum ImportsAction {
    List {
        /// The path to the file
        path: PathBuf,
    },
}

#[derive(clap::Args)]
struct WorkspaceArgs {
    #[command(subcommand)]
    command: WorkspaceCommand,
}

#[derive(Subcommand)]
enum WorkspaceCommand {
    /// List the workspaces in the project
    List {
        #[arg(long)]
        page: Option<usize>,

        #[arg(long)]
        per_page: Option<usize>,
    },
    Module(WorkspaceModuleArgs),
}

#[derive(clap::Args)]
struct WorkspaceModuleArgs {
    #[command(subcommand)]
    command: WorkspaceModuleCommand,
}

#[derive(Subcommand)]
enum WorkspaceModuleCommand {
    /// List the modules in the workspace
    List {
        /// The path to the workspace
        path: PathBuf,

        #[arg(long)]
        page: Option<usize>,

        #[arg(long)]
        per_page: Option<usize>,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to list exports: {0}")]
    ListExports(#[from] query::ExportsError),

    #[error("Failed to print table: {0}")]
    PrintTable(#[from] std::io::Error),

    #[error("Failed to create project: {0}")]
    CreateProject(#[from] query::project::Error),

    #[error("Failed to find workspace by path: {0}")]
    WorkspaceNotFound(PathBuf),

    #[error("Failed to find module by path: {0}")]
    ModuleNotFound(PathBuf),
}

pub fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => match command {
            Command::Module(module_args) => run_module_args(module_args),
            Command::Parse(parse_args) => run_parse_args(parse_args),
            Command::Workspace(workspaces_args) => run_workspace_args(workspaces_args),
        },
        None => Ok(()),
    }
}

fn run_module_args(args: ModuleArgs) -> Result<(), Error> {
    let ModuleCommand::Export {
        command: ModuleExportCommand::List { path },
    } = args.command;
    let module_path = path
        .canonicalize()
        .map_err(|_| Error::ModuleNotFound(path))?;
    let directory = std::env::current_dir()?.canonicalize()?;
    let project = query::Project::new(vec![directory])?;
    let graph = &project.graph;
    let module_id = graph
        .find_module_by_path(&module_path)
        .ok_or(Error::ModuleNotFound(module_path))?;

    let mut exports = graph
        .module_exports(&module_id)
        .map(|id| graph.module_export(&id))
        .collect::<Vec<_>>();
    exports.sort_by(|a, b| {
        a.binding
            .cmp(&b.binding)
            .then_with(|| a.span.start.line.cmp(&b.span.start.line))
    });

    let table = exports
        .iter()
        .map(|export| {
            vec![
                export.binding.as_str().cell(),
                match export.export_kind {
                    ImportOrExportKind::Value => "value".cell(),
                    ImportOrExportKind::Type => "type".cell(),
                },
                export.span.start.line.to_string().cell(),
            ]
        })
        .collect::<Vec<_>>()
        .table()
        .title(vec!["Export".cell(), "Export kind".cell(), "Line".cell()]);

    print_stdout(table)?;
    Ok(())
}

fn run_parse_args(args: ParseArgs) -> Result<(), Error> {
    match args.command {
        ParseCommand::Exports {
            action: export_action,
        } => match export_action {
            ExportsAction::List { path } => {
                let exports = list_exports(path)?;
                let table = exports
                    .iter()
                    .map(|export| match export {
                        query::Export::Default(default_export) => vec![
                            "default".cell(),
                            "value".cell(),
                            default_export.span.start.line.to_string().cell(),
                        ],
                        query::Export::Named(named_export) => {
                            vec![
                                named_export.binding.clone().cell(),
                                match &named_export.export_kind {
                                    query::ImportOrExportKind::Value => "value".cell(),
                                    query::ImportOrExportKind::Type => "type".cell(),
                                },
                                named_export.span.start.line.to_string().cell(),
                            ]
                        }
                    })
                    .collect::<Vec<_>>()
                    .table()
                    .title(vec!["Export".cell(), "Export kind".cell(), "Line".cell()]);

                print_stdout(table)?;

                Ok(())
            }
        },
        ParseCommand::Imports { action } => match action {
            ImportsAction::List { path } => {
                let imports = query::list_imports(path)?;
                let table = imports
                    .iter()
                    .map(|import| match import {
                        query::Import::Default(default_import) => vec![
                            "default".cell(),
                            match &default_import.import_kind {
                                query::ImportOrExportKind::Value => "value".cell(),
                                query::ImportOrExportKind::Type => "type".cell(),
                            },
                            default_import.source.clone().cell(),
                            default_import.span.start.line.to_string().cell(),
                        ],
                        query::Import::Named(named_import) => vec![
                            named_import.imported.clone().cell(),
                            match &named_import.import_kind {
                                query::ImportOrExportKind::Value => "value".cell(),
                                query::ImportOrExportKind::Type => "type".cell(),
                            },
                            named_import.source.clone().cell(),
                            named_import.span.start.line.to_string().cell(),
                        ],
                        query::Import::Namespace(namespace_import) => vec![
                            "*".cell(),
                            match &namespace_import.import_kind {
                                query::ImportOrExportKind::Value => "value".cell(),
                                query::ImportOrExportKind::Type => "type".cell(),
                            },
                            namespace_import.source.clone().cell(),
                            namespace_import.span.start.line.to_string().cell(),
                        ],
                    })
                    .collect::<Vec<_>>()
                    .table()
                    .title(vec![
                        "Imported".cell(),
                        "Kind".cell(),
                        "Source".cell(),
                        "Line".cell(),
                    ]);

                print_stdout(table)?;

                Ok(())
            }
        },
    }
}

fn run_workspace_args(args: WorkspaceArgs) -> Result<(), Error> {
    match args.command {
        WorkspaceCommand::List { page, per_page } => {
            let directory = std::env::current_dir()?;
            let project = query::Project::new(vec![directory])?;
            let mut workspaces = project.graph.workspaces().collect::<Vec<WorkspaceId>>();

            workspaces.sort_by(|a, b| {
                let a_workspace = project.graph.workspace(a);
                let b_workspace = project.graph.workspace(b);
                a_workspace.path.cmp(&b_workspace.path)
            });

            if let Some(page) = page {
                let per_page = per_page.unwrap_or(10);
                let start = ((page - 1) * per_page).min(workspaces.len());
                let end = (start + per_page).min(workspaces.len());
                workspaces = workspaces[start..end].to_vec();
            }

            let table = workspaces
                .iter()
                .map(|workspace_id| {
                    let source_id = project.graph.workspace_source(workspace_id);
                    let source = project.graph.source(&source_id);
                    let workspace = project.graph.workspace(workspace_id);

                    if source.path == workspace.path {
                        vec![workspace.path.display().to_string().cell()]
                    } else {
                        let relative = workspace
                            .path
                            .strip_prefix(&source.path)
                            .unwrap_or(&workspace.path);
                        vec![relative.display().to_string().cell()]
                    }
                })
                .collect::<Vec<_>>()
                .table()
                .title(vec!["Directory".cell()]);

            print_stdout(table)?;

            Ok(())
        }
        WorkspaceCommand::Module(workspace_module_args) => {
            run_workspace_module_args(workspace_module_args)
        }
    }
}

fn run_workspace_module_args(args: WorkspaceModuleArgs) -> Result<(), Error> {
    match args.command {
        WorkspaceModuleCommand::List {
            path,
            page,
            per_page,
        } => {
            let directory = std::env::current_dir()?.canonicalize()?;
            let project = query::Project::new(vec![&directory])?;
            let workspace_directory = path.canonicalize()?;
            let workspace_id = project
                .graph
                .find_workspace_by_path(&workspace_directory)
                .ok_or_else(|| Error::WorkspaceNotFound(workspace_directory.clone()))?;

            let mut modules = project
                .graph
                .workspace_modules(&workspace_id)
                .collect::<Vec<_>>();

            modules.sort_by(|a, b| {
                let a_module = project.graph.module(a);
                let b_module = project.graph.module(b);
                a_module.path.cmp(&b_module.path)
            });

            if let Some(page) = page {
                let per_page = per_page.unwrap_or(10);
                let start = ((page - 1) * per_page).min(modules.len());
                let end = (start + per_page).min(modules.len());
                modules = modules[start..end].to_vec();
            }

            let table = modules
                .iter()
                .map(|module_id| {
                    let module = project.graph.module(module_id);
                    let relative = module.path.strip_prefix(&directory).unwrap_or(&module.path);
                    vec![relative.display().to_string().cell()]
                })
                .collect::<Vec<_>>()
                .table()
                .title(vec!["Module".cell()]);

            print_stdout(table)?;

            Ok(())
        }
    }
}
