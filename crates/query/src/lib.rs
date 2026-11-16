use std::path::Path;

use oxc_span::{SourceType, UnknownExtension};
use parse::parse;
pub use parse::{Export, Import, ImportOrExportKind, ParseError};

mod parse;

#[derive(thiserror::Error, Debug)]
pub enum ExportsError {
    #[error("Unknown extension: {0}")]
    UnknownExtension(#[from] UnknownExtension),

    #[error("Parse error: {0}")]
    ParseError(#[from] parse::ParseError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub fn list_exports<P: AsRef<Path>>(path: P) -> Result<Vec<Export>, ExportsError> {
    let contents = std::fs::read_to_string(&path)?;
    let allocator = oxc_allocator::Allocator::default();
    let source_type = SourceType::from_path(&path)?;
    let result = parse(&allocator, &contents, source_type)?;

    Ok(result.exports.into_values().collect())
}

pub fn list_imports<P: AsRef<Path>>(path: P) -> Result<Vec<parse::Import>, ExportsError> {
    let contents = std::fs::read_to_string(&path)?;
    let allocator = oxc_allocator::Allocator::default();
    let source_type = SourceType::from_path(&path)?;
    let result = parse(&allocator, &contents, source_type)?;

    Ok(result.imports)
}
