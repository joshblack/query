use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cli_table::{Cell, Table, print_stdout};
use query::list_exports;

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
        action: ParseAction,
    },
}

#[derive(Subcommand)]
enum ParseAction {
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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to list exports: {0}")]
    ListExports(#[from] query::ExportsError),

    #[error("Failed to print table: {0}")]
    PrintTable(#[from] std::io::Error),
}

pub fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => match command {
            Command::Parse {
                action: parse_action,
            } => match parse_action {
                ParseAction::Exports {
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
                ParseAction::Imports { action } => match action {
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
            },
        },
        None => Ok(()),
    }
}
