use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cli_table::{Cell, Table, print_stdout};
use query::{list_exports, project::WorkspaceId};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Parse {
        #[command(subcommand)]
        command: ParseCommand,
    },
    Workspaces(WorkspacesArgs),
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
struct WorkspacesArgs {
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
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to list exports: {0}")]
    ListExports(#[from] query::ExportsError),

    #[error("Failed to print table: {0}")]
    PrintTable(#[from] std::io::Error),

    #[error("Failed to create project: {0}")]
    CreateProject(#[from] query::project::Error),
}

pub fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => match command {
            Command::Parse { command: action } => run_parse_command(action),
            Command::Workspaces(workspaces_args) => run_workspaces_args(workspaces_args),
        },
        None => Ok(()),
    }
}

fn run_parse_command(action: ParseCommand) -> Result<(), Error> {
    match action {
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

fn run_workspaces_args(args: WorkspacesArgs) -> Result<(), Error> {
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
                    let source_id = project.graph.workspace_source(&workspace_id);
                    let source = project.graph.source(&source_id);
                    let workspace = project.graph.workspace(&workspace_id);

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
    }
}
