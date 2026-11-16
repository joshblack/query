#![allow(unused)]

use std::collections::{HashMap, hash_map::Entry};

use line_index::LineIndex;
use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Declaration, ImportDeclarationSpecifier};
use oxc_ast_visit::Visit;
use oxc_diagnostics::Diagnostics;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

pub fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
) -> Result<Visitor, ParseError> {
    let parser_ret = Parser::new(allocator, source, source_type).parse();
    if !parser_ret.diagnostics.is_empty() {
        return Err(ParseError::SyntaxError(parser_ret.diagnostics));
    }

    let semantic_ret = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(&parser_ret.program);
    if !semantic_ret.diagnostics.is_empty() {
        return Err(ParseError::SyntaxError(semantic_ret.diagnostics));
    }

    let mut visitor = Visitor::new(source);
    visitor.visit_program(&parser_ret.program);

    Ok(visitor)
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Syntax error: {0:?}")]
    SyntaxError(Diagnostics),
}

#[derive(Debug)]
pub struct Visitor {
    errors: Vec<VisitError>,
    pub exports: HashMap<ExportKey, Export>,
    imports: Vec<Import>,
    line_index: LineIndex,
    pending_exports: Vec<PendingExport>,
}

impl Visitor {
    fn new(source: &str) -> Self {
        Self {
            errors: Default::default(),
            exports: Default::default(),
            imports: Default::default(),
            line_index: LineIndex::new(source),
            pending_exports: Default::default(),
        }
    }

    fn insert_export(&mut self, export: Export) {
        match self.exports.entry(ExportKey::from(&export)) {
            Entry::Occupied(occupied_entry) => {
                self.errors
                    .push(VisitError::DuplicateExport(occupied_entry.key().clone()));
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(export);
            }
        }
    }
}

impl<'a> Visit<'a> for Visitor {
    // export * from 'mod'
    // export * as ns from 'mod'
    fn visit_export_all_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportAllDeclaration<'a>,
    ) {
        let Some(source) = &declaration.source.value.as_str() else {
            self.errors
                .push(VisitError::InvalidSource(Span::from_oxc_span(
                    &self.line_index,
                    declaration.span,
                )));
            return;
        };

        match &declaration.exported {
            Some(exported) => {
                let binding = exported.name().to_string();
                self.pending_exports
                    .push(PendingExport::Namespace(PendingNamespaceExport {
                        export_kind: declaration.export_kind.into(),
                        exported: binding,
                        source: source.to_string(),
                        span: Span::from_oxc_span(&self.line_index, declaration.span),
                    }));
            }
            None => {
                self.pending_exports
                    .push(PendingExport::All(PendingAllExport {
                        export_kind: declaration.export_kind.into(),
                        source: source.to_string(),
                        span: Span::from_oxc_span(&self.line_index, declaration.span),
                    }));
            }
        }
    }

    fn visit_export_declaration(&mut self, statement: &oxc_ast::ast::ExportDeclaration<'a>) {
        match &statement.declaration {
            Declaration::VariableDeclaration(variable_declaration) => {
                for declarator in &variable_declaration.declarations {
                    match &declarator.id {
                        BindingPattern::BindingIdentifier(binding_identifier) => {
                            let binding = binding_identifier.name.to_string();
                            self.insert_export(Export::Named(NamedExport {
                                binding,
                                span: Span::from_oxc_span(&self.line_index, declarator.span),
                                export_kind: ImportOrExportKind::Value,
                            }));
                        }
                        BindingPattern::ObjectPattern(object_pattern) => todo!(),
                        BindingPattern::ArrayPattern(array_pattern) => todo!(),
                        BindingPattern::AssignmentPattern(assignment_pattern) => todo!(),
                    }
                }
            }
            Declaration::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    let binding = id.name.to_string();
                    self.insert_export(Export::Named(NamedExport {
                        binding,
                        span: Span::from_oxc_span(&self.line_index, function.span),
                        export_kind: ImportOrExportKind::Value,
                    }));
                }
            }
            Declaration::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    let binding = id.name.to_string();
                    self.insert_export(Export::Named(NamedExport {
                        binding,
                        span: Span::from_oxc_span(&self.line_index, class.span),
                        export_kind: ImportOrExportKind::Value,
                    }));
                }
            }
            Declaration::TSTypeAliasDeclaration(tstype_alias_declaration) => {
                let binding = tstype_alias_declaration.id.name.to_string();
                self.insert_export(Export::Named(NamedExport {
                    binding,
                    span: Span::from_oxc_span(&self.line_index, tstype_alias_declaration.span),
                    export_kind: ImportOrExportKind::Type,
                }));
            }
            Declaration::TSInterfaceDeclaration(tsinterface_declaration) => {
                let binding = tsinterface_declaration.id.name.to_string();
                self.insert_export(Export::Named(NamedExport {
                    binding,
                    span: Span::from_oxc_span(&self.line_index, tsinterface_declaration.span),
                    export_kind: ImportOrExportKind::Type,
                }));
            }
            Declaration::TSEnumDeclaration(_) => {}
            Declaration::TSExternalModuleDeclaration(_) => {}
            Declaration::TSNamespaceDeclaration(_) => {}
            Declaration::TSGlobalDeclaration(_) => {}
            Declaration::TSImportEqualsDeclaration(_) => {}
        }
    }

    fn visit_export_default_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        self.insert_export(Export::Default(DefaultExport {
            span: Span::from_oxc_span(&self.line_index, declaration.span),
        }));
    }

    // export { a, type b } from 'mod'
    // export type { c } from 'mod'
    fn visit_export_from_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportFromDeclaration<'a>,
    ) {
        let Some(source) = &declaration.source.value.as_str() else {
            self.errors
                .push(VisitError::InvalidSource(Span::from_oxc_span(
                    &self.line_index,
                    declaration.source.span,
                )));
            return;
        };

        for specifier in &declaration.specifiers {
            self.pending_exports
                .push(PendingExport::Named(PendingNamedExport {
                    export_kind: match declaration.export_kind {
                        oxc_ast::ast::ImportOrExportKind::Type => ImportOrExportKind::Type,
                        oxc_ast::ast::ImportOrExportKind::Value => specifier.export_kind.into(),
                    },
                    exported: specifier.exported.name().to_string(),
                    local: specifier.local.name().to_string(),
                    source: source.to_string(),
                    span: Span::from_oxc_span(&self.line_index, specifier.span),
                }));
        }
    }

    fn visit_export_named_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        for specifier in &declaration.specifiers {
            let binding = specifier.exported.name().to_string();
            let export_kind = match declaration.export_kind {
                oxc_ast::ast::ImportOrExportKind::Type => ImportOrExportKind::Type,
                oxc_ast::ast::ImportOrExportKind::Value => specifier.export_kind.into(),
            };
            self.insert_export(Export::Named(NamedExport {
                binding,
                span: Span::from_oxc_span(&self.line_index, specifier.span),
                export_kind,
            }));
        }
    }

    fn visit_import_declaration(&mut self, declaration: &oxc_ast::ast::ImportDeclaration<'a>) {
        let Some(source) = &declaration.source.value.as_str() else {
            self.errors
                .push(VisitError::InvalidSource(Span::from_oxc_span(
                    &self.line_index,
                    declaration.source.span,
                )));
            return;
        };

        for specifier in declaration.specifiers.iter().flatten() {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                    self.imports.push(Import::Named(NamedImport {
                        import_kind: match declaration.import_kind {
                            oxc_ast::ast::ImportOrExportKind::Type => ImportOrExportKind::Type,
                            oxc_ast::ast::ImportOrExportKind::Value => {
                                import_specifier.import_kind.into()
                            }
                        },
                        imported: import_specifier.imported.name().to_string(),
                        local: import_specifier.local.name.to_string(),
                        span: Span::from_oxc_span(&self.line_index, import_specifier.span),
                        source: source.to_string(),
                    }));
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(import_default_specifier) => {
                    self.imports.push(Import::Default(DefaultImport {
                        import_kind: declaration.import_kind.into(),
                        span: Span::from_oxc_span(&self.line_index, import_default_specifier.span),
                        source: source.to_string(),
                    }));
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                    import_namespace_specifier,
                ) => {
                    self.imports.push(Import::Namespace(NamespaceImport {
                        local: import_namespace_specifier.local.name.to_string(),
                        span: Span::from_oxc_span(
                            &self.line_index,
                            import_namespace_specifier.span,
                        ),
                        source: source.to_string(),
                    }));
                }
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum VisitError {
    #[error("Duplicate export found: {0:?}")]
    DuplicateExport(ExportKey),

    #[error("Invalid source: {0:?}")]
    InvalidSource(Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportOrExportKind {
    Type,
    Value,
}

impl From<oxc_ast::ast::ImportOrExportKind> for ImportOrExportKind {
    fn from(import_kind: oxc_ast::ast::ImportOrExportKind) -> Self {
        match import_kind {
            oxc_ast::ast::ImportOrExportKind::Type => ImportOrExportKind::Type,
            oxc_ast::ast::ImportOrExportKind::Value => ImportOrExportKind::Value,
        }
    }
}

#[derive(Debug)]
enum Import {
    Default(DefaultImport),
    Named(NamedImport),
    Namespace(NamespaceImport),
}

#[derive(Debug)]
struct DefaultImport {
    import_kind: ImportOrExportKind,
    source: String,
    span: Span,
}

#[derive(Debug)]
struct NamedImport {
    import_kind: ImportOrExportKind,
    imported: String,
    local: String,
    source: String,
    span: Span,
}

#[derive(Debug)]
struct NamespaceImport {
    local: String,
    source: String,
    span: Span,
}

#[derive(Debug)]
pub enum Export {
    Default(DefaultExport),
    Named(NamedExport),
}

#[derive(Debug)]
pub struct DefaultExport {
    pub span: Span,
}

#[derive(Debug)]
pub struct NamedExport {
    pub binding: String,
    pub span: Span,
    pub export_kind: ImportOrExportKind,
}

type ExportKey = (String, ImportOrExportKind);

impl From<&Export> for ExportKey {
    fn from(export: &Export) -> Self {
        match export {
            Export::Default(_) => ("default".to_string(), ImportOrExportKind::Value),
            Export::Named(NamedExport {
                binding,
                export_kind,
                ..
            }) => (binding.clone(), *export_kind),
            // Export::Namespace => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum PendingExport {
    // export * from 'mod'
    All(PendingAllExport),
    // export { a, type b } from 'mod'
    // export type { c } from 'mod'
    Named(PendingNamedExport),
    // export * as ns from 'mod'
    Namespace(PendingNamespaceExport),
}

#[derive(Debug)]
pub struct PendingAllExport {
    pub export_kind: ImportOrExportKind,
    pub source: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct PendingNamedExport {
    pub export_kind: ImportOrExportKind,
    pub exported: String,
    pub local: String,
    pub source: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct PendingNamespaceExport {
    pub export_kind: ImportOrExportKind,
    pub exported: String,
    pub source: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    fn from_offset(index: &LineIndex, offset: u32) -> Self {
        let location = index
            .to_wide(
                line_index::WideEncoding::Utf16,
                index.line_col(offset.into()),
            )
            .expect("Oxc span offsets should be valid source positions");

        Self {
            line: location.line as usize + 1,
            column: location.col as usize + 1,
        }
    }
}

#[derive(Debug)]
pub struct Span {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl Span {
    fn from_oxc_span(index: &LineIndex, span: oxc_span::Span) -> Self {
        Self {
            start: SourceLocation::from_offset(index, span.start),
            end: SourceLocation::from_offset(index, span.end),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn test_no_export() {
        let allocator = Allocator::default();
        let source = "const x = 1";
        let source_type = SourceType::default();
        let result = parse(&allocator, source, source_type).unwrap();
        assert_eq!(result.exports.len(), 0);
    }

    #[test]
    fn test_syntax_error() {
        let allocator = Allocator::default();
        let source = "export default";
        let source_type = SourceType::default();
        let result = parse(&allocator, source, source_type);

        assert_matches!(result, Err(ParseError::SyntaxError(_)));
    }

    #[test]
    fn test_default_export() {
        let allocator = Allocator::default();
        let source = "export default 1";
        let source_type = SourceType::default();
        let result = parse(&allocator, source, source_type).unwrap();
        let key = ("default".to_string(), ImportOrExportKind::Value);
        assert!(result.exports.contains_key(&key));
    }

    #[test]
    fn test_named_export() {
        let allocator = Allocator::default();
        let source = r#"
            const t1 = 1;
            type t2 = string;
            export {
                // ExportSpecifier
                t1,
                // ExportSpecifier with local type
                type t2,
                // ExportSpecifier with different exported than local
                t1 as t3,
                // ExportSpecifier with different exported than local and local type
                type t2 as t4,
            };

            type t5 = string;
            // ExportSpecifier with type on declaration
            export type { t5 };
        "#;
        let source_type = SourceType::ts();
        let result = parse(&allocator, source, source_type).unwrap();

        assert!(
            result
                .exports
                .contains_key(&("t1".to_string(), ImportOrExportKind::Value))
        );
        assert!(
            result
                .exports
                .contains_key(&("t2".to_string(), ImportOrExportKind::Type))
        );
        assert!(
            result
                .exports
                .contains_key(&("t3".to_string(), ImportOrExportKind::Value))
        );
        assert!(
            result
                .exports
                .contains_key(&("t4".to_string(), ImportOrExportKind::Type))
        );
        assert!(
            result
                .exports
                .contains_key(&("t5".to_string(), ImportOrExportKind::Type))
        );
    }

    #[test]
    fn test_export_declaration() {
        let allocator = Allocator::default();
        let source = r#"
            export const t1 = 1;
            export function t2() {}
            export class t3 {}
            export type t4 = string;
            export interface t5 {}
        "#;
        let source_type = SourceType::ts();
        let result = parse(&allocator, source, source_type).unwrap();

        assert!(
            result
                .exports
                .contains_key(&("t1".to_string(), ImportOrExportKind::Value))
        );
        assert!(
            result
                .exports
                .contains_key(&("t2".to_string(), ImportOrExportKind::Value))
        );
        assert!(
            result
                .exports
                .contains_key(&("t3".to_string(), ImportOrExportKind::Value))
        );
        assert!(
            result
                .exports
                .contains_key(&("t4".to_string(), ImportOrExportKind::Type))
        );
        assert!(
            result
                .exports
                .contains_key(&("t5".to_string(), ImportOrExportKind::Type))
        );
    }

    #[test]
    fn test_export_from_declaration() {
        let allocator = Allocator::default();
        let source = r#"
            export * from 'mod1';
            export * as ns from 'mod2';
            export { a, type b } from 'mod3';
            export type { c } from 'mod4';
        "#;
        let source_type = SourceType::ts();
        let result = parse(&allocator, source, source_type).unwrap();

        assert_eq!(result.pending_exports.len(), 5);
        assert!(result.pending_exports.iter().any(|export| {
            match export {
                PendingExport::All(PendingAllExport {
                    export_kind,
                    source,
                    ..
                }) => source == "mod1" && export_kind == &ImportOrExportKind::Value,
                _ => false,
            }
        }));
        assert!(result.pending_exports.iter().any(|export| {
            match export {
                PendingExport::Namespace(PendingNamespaceExport {
                    exported,
                    source,
                    export_kind,
                    ..
                }) => {
                    exported == "ns"
                        && source == "mod2"
                        && export_kind == &ImportOrExportKind::Value
                }
                _ => false,
            }
        }));
        assert!(result.pending_exports.iter().any(|export| {
            match export {
                PendingExport::Named(PendingNamedExport {
                    exported,
                    local,
                    source,
                    export_kind,
                    ..
                }) => {
                    exported == "a"
                        && local == "a"
                        && source == "mod3"
                        && export_kind == &ImportOrExportKind::Value
                }
                _ => false,
            }
        }));
        assert!(result.pending_exports.iter().any(|export| {
            match export {
                PendingExport::Named(PendingNamedExport {
                    exported,
                    local,
                    source,
                    export_kind,
                    ..
                }) => {
                    exported == "b"
                        && local == "b"
                        && source == "mod3"
                        && export_kind == &ImportOrExportKind::Type
                }
                _ => false,
            }
        }));
        assert!(result.pending_exports.iter().any(|export| {
            match export {
                PendingExport::Named(PendingNamedExport {
                    exported,
                    local,
                    source,
                    export_kind,
                    ..
                }) => {
                    exported == "c"
                        && local == "c"
                        && source == "mod4"
                        && export_kind == &ImportOrExportKind::Type
                }
                _ => false,
            }
        }));
    }

    #[test]
    fn test_export_all_declaration() {
        let allocator = Allocator::default();
        let source = r#"
            export * from 'mod1';
            export * as ns1 from 'mod2';
            export type * as ns2 from 'mod2';
        "#;
        let source_type = SourceType::ts();
        let result = parse(&allocator, source, source_type).unwrap();

        assert_eq!(result.pending_exports.len(), 3);
        assert!(result.pending_exports.iter().any(|export| {
            match export {
                PendingExport::All(PendingAllExport {
                    export_kind,
                    source,
                    ..
                }) => source == "mod1" && export_kind == &ImportOrExportKind::Value,
                _ => false,
            }
        }));
        assert!(result.pending_exports.iter().any(|export| {
            match export {
                PendingExport::Namespace(PendingNamespaceExport {
                    exported,
                    source,
                    export_kind,
                    ..
                }) => {
                    exported == "ns1"
                        && source == "mod2"
                        && export_kind == &ImportOrExportKind::Value
                }
                _ => false,
            }
        }));
        assert!(result.pending_exports.iter().any(|export| {
            match export {
                PendingExport::Namespace(PendingNamespaceExport {
                    exported,
                    source,
                    export_kind,
                    ..
                }) => {
                    exported == "ns2"
                        && source == "mod2"
                        && export_kind == &ImportOrExportKind::Type
                }
                _ => false,
            }
        }));
    }

    #[test]
    fn test_duplicate_export_error() {
        let allocator = Allocator::default();
        let source = r#"
            const t1 = 1;
            const t2 = 2;

            export { t1 };
            export { t2 as t1 };
        "#;
        let source_type = SourceType::ts();
        let result = parse(&allocator, source, source_type).unwrap();

        assert_eq!(result.errors.len(), 1);
        assert_matches!(
            &result.errors[0],
            VisitError::DuplicateExport((binding, _)) if binding == "t1"
        );
    }

    #[test]
    fn test_import_default() {
        let allocator = Allocator::default();
        let source = r#"
            import t from "module-name";
        "#;
        let source_type = SourceType::ts();
        let result = parse(&allocator, source, source_type).unwrap();

        assert_eq!(result.imports.len(), 1);
        assert_matches!(
            &result.imports[0],
            Import::Default(DefaultImport { source, .. }) if source == "module-name"
        );
    }

    #[test]
    fn test_import_named() {
        let allocator = Allocator::default();
        let source = r#"
            import { t1, t2 } from 'mod';
            import type { t3 } from 'mod';
            import { t1 as t4 } from 'mod';
            import { type t5 } from 'mod';
        "#;
        let source_type = SourceType::ts();
        let result = parse(&allocator, source, source_type).unwrap();

        assert!(result.imports.iter().any(|import| {
            match import {
                Import::Default(_) => false,
                Import::Named(named_import) => {
                    named_import.imported == "t1"
                        && named_import.local == "t1"
                        && named_import.source == "mod"
                        && named_import.import_kind == ImportOrExportKind::Value
                }
                Import::Namespace(_) => false,
            }
        }));

        assert!(result.imports.iter().any(|import| {
            match import {
                Import::Default(_) => false,
                Import::Named(named_import) => {
                    named_import.imported == "t2"
                        && named_import.local == "t2"
                        && named_import.source == "mod"
                        && named_import.import_kind == ImportOrExportKind::Value
                }
                Import::Namespace(_) => false,
            }
        }));

        assert!(result.imports.iter().any(|import| {
            match import {
                Import::Default(_) => false,
                Import::Named(named_import) => {
                    named_import.imported == "t3"
                        && named_import.local == "t3"
                        && named_import.source == "mod"
                        && named_import.import_kind == ImportOrExportKind::Type
                }
                Import::Namespace(_) => false,
            }
        }));

        assert!(result.imports.iter().any(|import| {
            match import {
                Import::Default(_) => false,
                Import::Named(named_import) => {
                    named_import.imported == "t1"
                        && named_import.local == "t4"
                        && named_import.source == "mod"
                        && named_import.import_kind == ImportOrExportKind::Value
                }
                Import::Namespace(_) => false,
            }
        }));

        assert!(result.imports.iter().any(|import| {
            match import {
                Import::Default(_) => false,
                Import::Named(named_import) => {
                    named_import.imported == "t5"
                        && named_import.local == "t5"
                        && named_import.source == "mod"
                        && named_import.import_kind == ImportOrExportKind::Type
                }
                Import::Namespace(_) => false,
            }
        }));
    }

    #[test]
    fn test_import_namespace() {
        let allocator = Allocator::default();
        let source = r#"
            import * as ns from 'mod';
            import type * as ns2 from 'mod';
        "#;
        let source_type = SourceType::ts();
        let result = parse(&allocator, source, source_type).unwrap();

        assert!(result.imports.iter().any(|import| {
            match import {
                Import::Namespace(namespace_import) => {
                    namespace_import.local == "ns" && namespace_import.source == "mod"
                }
                _ => false,
            }
        }));

        assert!(result.imports.iter().any(|import| {
            match import {
                Import::Namespace(namespace_import) => {
                    namespace_import.local == "ns2" && namespace_import.source == "mod"
                }
                _ => false,
            }
        }));
    }
}
