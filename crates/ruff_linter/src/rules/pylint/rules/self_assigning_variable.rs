use itertools::Itertools;
use ruff_python_ast::{self as ast, Expr};

use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_text_size::Ranged;

use crate::Violation;
use crate::checkers::ast::Checker;

/// ## What it does
/// Checks for self-assignment of variables.
///
/// ## Why is this bad?
/// Self-assignment of variables is redundant and likely a mistake.
///
/// ## Example
/// ```python
/// country = "Poland"
/// country = country
/// ```
///
/// Use instead:
/// ```python
/// country = "Poland"
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SelfAssigningVariable {
    name: String,
}

impl Violation for SelfAssigningVariable {
    #[derive_message_formats]
    fn message(&self) -> String {
        let SelfAssigningVariable { name } = self;
        format!("Self-assignment of variable `{name}`")
    }
}

/// PLW0127
pub(crate) fn self_assignment(checker: &Checker, assign: &ast::StmtAssign) {
    // Assignments in class bodies are attributes (e.g., `x = x` assigns `x` to `self.x`, and thus
    // is not a self-assignment).
    if checker.semantic().current_scope().kind.is_class() {
        return;
    }

    for (left, right) in assign
        .targets
        .iter()
        .chain(std::iter::once(assign.value.as_ref()))
        .tuple_combinations()
    {
        visit_assignments(checker, left, right);
    }
}

/// PLW0127
pub(crate) fn self_annotated_assignment(checker: &Checker, assign: &ast::StmtAnnAssign) {
    let Some(value) = assign.value.as_ref() else {
        return;
    };

    // Assignments in class bodies are attributes (e.g., `x = x` assigns `x` to `self.x`, and thus
    // is not a self-assignment).
    if checker.semantic().current_scope().kind.is_class() {
        return;
    }

    visit_assignments(checker, &assign.target, value);
}

fn visit_assignments(checker: &Checker, left: &Expr, right: &Expr) {
    match (left, right) {
        (Expr::Tuple(lhs), Expr::Tuple(rhs)) if lhs.len() == rhs.len() => lhs
            .iter()
            .zip(rhs)
            .for_each(|(lhs_elem, rhs_elem)| visit_assignments(checker, lhs_elem, rhs_elem)),
        (
            Expr::Name(ast::ExprName { id: lhs_name, .. }),
            Expr::Name(ast::ExprName { id: rhs_name, .. }),
        ) if lhs_name == rhs_name => {
            checker.report_diagnostic(
                SelfAssigningVariable {
                    name: lhs_name.to_string(),
                },
                left.range(),
            );
        }
        _ => {}
    }
}
