use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_ast::visitor::Visitor;
use ruff_python_ast::{self as ast, Expr, Int, Number, StmtFor};
use ruff_python_semantic::SemanticModel;
use ruff_text_size::Ranged;

use crate::checkers::ast::Checker;
use crate::rules::pylint::helpers::SequenceIndexVisitor;
use crate::{AlwaysFixableViolation, Edit, Fix};

/// ## What it does
/// Checks for index-based list accesses during `enumerate` iterations.
///
/// ## Why is this bad?
/// When iterating over a list with `enumerate`, the current item is already
/// available alongside its index. Using the index to look up the item is
/// unnecessary.
///
/// ## Example
/// ```python
/// letters = ["a", "b", "c"]
///
/// for index, letter in enumerate(letters):
///     print(letters[index])
/// ```
///
/// Use instead:
/// ```python
/// letters = ["a", "b", "c"]
///
/// for index, letter in enumerate(letters):
///     print(letter)
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct UnnecessaryListIndexLookup;

impl AlwaysFixableViolation for UnnecessaryListIndexLookup {
    #[derive_message_formats]
    fn message(&self) -> String {
        "List index lookup in `enumerate()` loop".to_string()
    }

    fn fix_title(&self) -> String {
        "Use the loop variable directly".to_string()
    }
}

/// PLR1736
pub(crate) fn unnecessary_list_index_lookup(checker: &Checker, stmt_for: &StmtFor) {
    let Some((sequence, index_name, value_name)) =
        enumerate_items(&stmt_for.iter, &stmt_for.target, checker.semantic())
    else {
        return;
    };

    let ranges = {
        let mut visitor = SequenceIndexVisitor::new(&sequence.id, &index_name.id, &value_name.id);
        visitor.visit_body(&stmt_for.body);
        visitor.visit_body(&stmt_for.orelse);
        visitor.into_accesses()
    };

    for range in ranges {
        let mut diagnostic = checker.report_diagnostic(UnnecessaryListIndexLookup, range);
        diagnostic.set_fix(Fix::safe_edits(
            Edit::range_replacement(value_name.id.to_string(), range),
            [noop(index_name), noop(value_name)],
        ));
    }
}

/// PLR1736
pub(crate) fn unnecessary_list_index_lookup_comprehension(checker: &Checker, expr: &Expr) {
    let (Expr::Generator(ast::ExprGenerator {
        elt, generators, ..
    })
    | Expr::DictComp(ast::ExprDictComp {
        value: elt,
        generators,
        ..
    })
    | Expr::SetComp(ast::ExprSetComp {
        elt, generators, ..
    })
    | Expr::ListComp(ast::ExprListComp {
        elt, generators, ..
    })) = expr
    else {
        return;
    };

    for comp in generators {
        let Some((sequence, index_name, value_name)) =
            enumerate_items(&comp.iter, &comp.target, checker.semantic())
        else {
            return;
        };

        let ranges = {
            let mut visitor =
                SequenceIndexVisitor::new(&sequence.id, &index_name.id, &value_name.id);
            visitor.visit_expr(elt.as_ref());
            visitor.into_accesses()
        };

        for range in ranges {
            let mut diagnostic = checker.report_diagnostic(UnnecessaryListIndexLookup, range);
            diagnostic.set_fix(Fix::safe_edits(
                Edit::range_replacement(value_name.id.to_string(), range),
                [noop(index_name), noop(value_name)],
            ));
        }
    }
}

fn enumerate_items<'a>(
    call_expr: &'a Expr,
    tuple_expr: &'a Expr,
    semantic: &SemanticModel,
) -> Option<(&'a ast::ExprName, &'a ast::ExprName, &'a ast::ExprName)> {
    let ast::ExprCall {
        func, arguments, ..
    } = call_expr.as_call_expr()?;

    let Expr::Tuple(ast::ExprTuple { elts, .. }) = tuple_expr else {
        return None;
    };
    let [index, value] = elts.as_slice() else {
        return None;
    };

    // Grab the variable names.
    let Expr::Name(index_name) = index else {
        return None;
    };

    let Expr::Name(value_name) = value else {
        return None;
    };

    // If either of the variable names are intentionally ignored by naming them `_`, then don't
    // emit.
    if index_name.id == "_" || value_name.id == "_" {
        return None;
    }

    // Get the first argument of the enumerate call.
    let Some(Expr::Name(sequence)) = arguments.args.first() else {
        return None;
    };

    // If the `enumerate` call has a non-zero `start`, don't omit.
    if !arguments
        .find_argument_value("start", 1)
        .is_none_or(|expr| {
            matches!(
                expr,
                Expr::NumberLiteral(ast::ExprNumberLiteral {
                    value: Number::Int(Int::ZERO),
                    ..
                })
            )
        })
    {
        return None;
    }

    // Check that the function is the `enumerate` builtin.
    if !semantic.match_builtin_expr(func, "enumerate") {
        return None;
    }

    Some((sequence, index_name, value_name))
}

/// Return a no-op edit for the given name.
fn noop(name: &ast::ExprName) -> Edit {
    Edit::range_replacement(name.id.to_string(), name.range())
}
