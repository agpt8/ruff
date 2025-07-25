use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_parser::TokenKind;
use ruff_text_size::{Ranged, TextRange, TextSize};

use crate::checkers::ast::LintContext;
use crate::rules::pycodestyle::rules::logical_lines::{DefinitionState, LogicalLine};
use crate::{AlwaysFixableViolation, Edit, Fix};

/// ## What it does
/// Checks for missing whitespace around the equals sign in an unannotated
/// function keyword parameter.
///
/// ## Why is this bad?
/// According to [PEP 8], there should be no spaces around the equals sign in a
/// keyword parameter, if it is unannotated:
///
/// > Don’t use spaces around the = sign when used to indicate a keyword
/// > argument, or when used to indicate a default value for an unannotated
/// > function parameter.
///
/// ## Example
/// ```python
/// def add(a = 0) -> int:
///     return a + 1
/// ```
///
/// Use instead:
/// ```python
/// def add(a=0) -> int:
///     return a + 1
/// ```
///
/// [PEP 8]: https://peps.python.org/pep-0008/#whitespace-in-expressions-and-statements
#[derive(ViolationMetadata)]
pub(crate) struct UnexpectedSpacesAroundKeywordParameterEquals;

impl AlwaysFixableViolation for UnexpectedSpacesAroundKeywordParameterEquals {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Unexpected spaces around keyword / parameter equals".to_string()
    }

    fn fix_title(&self) -> String {
        "Remove whitespace".to_string()
    }
}

/// ## What it does
/// Checks for missing whitespace around the equals sign in an annotated
/// function keyword parameter.
///
/// ## Why is this bad?
/// According to [PEP 8], the spaces around the equals sign in a keyword
/// parameter should only be omitted when the parameter is unannotated:
///
/// > Don’t use spaces around the = sign when used to indicate a keyword
/// > argument, or when used to indicate a default value for an unannotated
/// > function parameter.
///
/// ## Example
/// ```python
/// def add(a: int=0) -> int:
///     return a + 1
/// ```
///
/// Use instead:
/// ```python
/// def add(a: int = 0) -> int:
///     return a + 1
/// ```
///
/// [PEP 8]: https://peps.python.org/pep-0008/#whitespace-in-expressions-and-statements
#[derive(ViolationMetadata)]
pub(crate) struct MissingWhitespaceAroundParameterEquals;

impl AlwaysFixableViolation for MissingWhitespaceAroundParameterEquals {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Missing whitespace around parameter equals".to_string()
    }

    fn fix_title(&self) -> String {
        "Add missing whitespace".to_string()
    }
}

/// E251, E252
pub(crate) fn whitespace_around_named_parameter_equals(line: &LogicalLine, context: &LintContext) {
    let mut parens = 0u32;
    let mut fstrings = 0u32;
    let mut annotated_func_arg = false;
    let mut prev_end = TextSize::default();

    let mut definition_state = DefinitionState::from_tokens(line.tokens());
    let mut iter = line.tokens().iter().peekable();

    while let Some(token) = iter.next() {
        let kind = token.kind();
        definition_state.visit_token_kind(kind);
        match kind {
            TokenKind::NonLogicalNewline => continue,
            TokenKind::FStringStart => fstrings += 1,
            TokenKind::FStringEnd => fstrings = fstrings.saturating_sub(1),
            TokenKind::Lpar | TokenKind::Lsqb => {
                parens = parens.saturating_add(1);
            }
            TokenKind::Rpar | TokenKind::Rsqb => {
                parens = parens.saturating_sub(1);
                if parens == 0 {
                    annotated_func_arg = false;
                }
            }
            TokenKind::Colon if parens == 1 && definition_state.in_function_definition() => {
                annotated_func_arg = true;
            }
            TokenKind::Comma if parens == 1 => {
                annotated_func_arg = false;
            }
            TokenKind::Equal
                if definition_state.in_type_params() || (parens > 0 && fstrings == 0) =>
            {
                if definition_state.in_type_params() || (annotated_func_arg && parens == 1) {
                    let start = token.start();
                    if start == prev_end && prev_end != TextSize::new(0) {
                        if let Some(mut diagnostic) = context.report_diagnostic_if_enabled(
                            MissingWhitespaceAroundParameterEquals,
                            token.range,
                        ) {
                            diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                                " ".to_string(),
                                token.start(),
                            )));
                        }
                    }

                    while let Some(next) = iter.peek() {
                        if next.kind() == TokenKind::NonLogicalNewline {
                            iter.next();
                        } else {
                            let next_start = next.start();

                            if next_start == token.end() {
                                if let Some(mut diagnostic) = context.report_diagnostic_if_enabled(
                                    MissingWhitespaceAroundParameterEquals,
                                    token.range,
                                ) {
                                    diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                                        " ".to_string(),
                                        token.end(),
                                    )));
                                }
                            }
                            break;
                        }
                    }
                } else {
                    // If there's space between the preceding token and the equals sign, report it.
                    if token.start() != prev_end {
                        if let Some(mut diagnostic) = context.report_diagnostic_if_enabled(
                            UnexpectedSpacesAroundKeywordParameterEquals,
                            TextRange::new(prev_end, token.start()),
                        ) {
                            diagnostic
                                .set_fix(Fix::safe_edit(Edit::deletion(prev_end, token.start())));
                        }
                    }

                    // If there's space between the equals sign and the following token, report it.
                    while let Some(next) = iter.peek() {
                        if next.kind() == TokenKind::NonLogicalNewline {
                            iter.next();
                        } else {
                            if next.start() != token.end() {
                                if let Some(mut diagnostic) = context.report_diagnostic_if_enabled(
                                    UnexpectedSpacesAroundKeywordParameterEquals,
                                    TextRange::new(token.end(), next.start()),
                                ) {
                                    diagnostic.set_fix(Fix::safe_edit(Edit::deletion(
                                        token.end(),
                                        next.start(),
                                    )));
                                }
                            }
                            break;
                        }
                    }
                }
            }
            _ => {}
        }

        prev_end = token.end();
    }
}
