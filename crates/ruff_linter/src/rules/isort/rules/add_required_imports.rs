use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_ast::helpers::is_docstring_stmt;
use ruff_python_ast::{self as ast, ModModule, PySourceType, Stmt};
use ruff_python_codegen::Stylist;
use ruff_python_parser::Parsed;
use ruff_python_semantic::{FutureImport, NameImport};
use ruff_text_size::{TextRange, TextSize};

use crate::Locator;
use crate::checkers::ast::LintContext;
use crate::importer::Importer;
use crate::settings::LinterSettings;
use crate::{AlwaysFixableViolation, Fix};

/// ## What it does
/// Adds any required imports, as specified by the user, to the top of the
/// file.
///
/// ## Why is this bad?
/// In some projects, certain imports are required to be present in all
/// files. For example, some projects assume that
/// `from __future__ import annotations` is enabled,
/// and thus require that import to be
/// present in all files. Omitting a "required" import (as specified by
/// the user) can cause errors or unexpected behavior.
///
/// ## Example
/// ```python
/// import typing
/// ```
///
/// Use instead:
/// ```python
/// from __future__ import annotations
///
/// import typing
/// ```
///
/// ## Options
/// - `lint.isort.required-imports`
#[derive(ViolationMetadata)]
pub(crate) struct MissingRequiredImport(pub String);

impl AlwaysFixableViolation for MissingRequiredImport {
    #[derive_message_formats]
    fn message(&self) -> String {
        let MissingRequiredImport(name) = self;
        format!("Missing required import: `{name}`")
    }

    fn fix_title(&self) -> String {
        let MissingRequiredImport(name) = self;
        format!("Insert required import: `{name}`")
    }
}

/// Return `true` if the [`Stmt`] includes the given [`AnyImportRef`].
fn includes_import(stmt: &Stmt, target: &NameImport) -> bool {
    match target {
        NameImport::Import(target) => {
            let Stmt::Import(ast::StmtImport {
                names,
                range: _,
                node_index: _,
            }) = &stmt
            else {
                return false;
            };
            names.iter().any(|alias| {
                alias.name == target.name.name
                    && alias.asname.as_deref() == target.name.as_name.as_deref()
            })
        }
        NameImport::ImportFrom(target) => {
            let Stmt::ImportFrom(ast::StmtImportFrom {
                module,
                names,
                level,
                range: _,
                node_index: _,
            }) = &stmt
            else {
                return false;
            };
            module.as_deref() == target.module.as_deref()
                && *level == target.level
                && names.iter().any(|alias| {
                    alias.name == target.name.name
                        && alias.asname.as_deref() == target.name.as_name.as_deref()
                })
        }
    }
}

fn add_required_import(
    required_import: &NameImport,
    parsed: &Parsed<ModModule>,
    locator: &Locator,
    stylist: &Stylist,
    source_type: PySourceType,
    context: &LintContext,
) {
    // Don't add imports to semantically-empty files.
    if parsed.suite().iter().all(is_docstring_stmt) {
        return;
    }

    // We don't need to add `__future__` imports to stubs.
    if source_type.is_stub() && required_import.is_future_import() {
        return;
    }

    // If the import is already present in a top-level block, don't add it.
    if parsed
        .suite()
        .iter()
        .any(|stmt| includes_import(stmt, required_import))
    {
        return;
    }

    // Always insert the diagnostic at top-of-file.
    let mut diagnostic = context.report_diagnostic(
        MissingRequiredImport(required_import.to_string()),
        TextRange::default(),
    );
    diagnostic.set_fix(Fix::safe_edit(
        Importer::new(parsed, locator, stylist).add_import(required_import, TextSize::default()),
    ));
}

/// I002
pub(crate) fn add_required_imports(
    parsed: &Parsed<ModModule>,
    locator: &Locator,
    stylist: &Stylist,
    settings: &LinterSettings,
    source_type: PySourceType,
    context: &LintContext,
) {
    for required_import in &settings.isort.required_imports {
        add_required_import(
            required_import,
            parsed,
            locator,
            stylist,
            source_type,
            context,
        );
    }
}
