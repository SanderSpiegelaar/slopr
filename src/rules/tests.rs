use tree_sitter::Node;

use crate::config::RuleSettings;
use crate::finding::Severity;
use crate::rules::{Check, Hit, Rule};
use crate::source::{Lang, SourceFile};

pub const RULES: &[Rule] = &[Rule {
    id: "test/tautological",
    summary: "Test assertion that always passes",
    default_severity: Severity::Error,
    langs: &[
        Lang::TypeScript,
        Lang::Tsx,
        Lang::JavaScript,
        Lang::Python,
        Lang::Rust,
    ],
    default_on: true,
    check: Check::File(tautological),
}];

fn tautological(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    for node in file.nodes() {
        let always_passes = match node.kind() {
            "call_expression" => ts_call(file, node),
            "call" => py_call(file, node),
            "assert_statement" => py_assert(file, node),
            "macro_invocation" => rs_macro(file, node),
            _ => false,
        };
        if always_passes && file.in_test(node) {
            hits.push(Hit {
                span: file.span(node.byte_range()),
                message: "assertion always passes".to_string(),
            });
        }
    }
}

fn ts_call(file: &SourceFile, call: Node<'_>) -> bool {
    let Some(callee) = call.child_by_field_name("function") else {
        return false;
    };
    let args = arguments(call.child_by_field_name("arguments"));
    match text(file, callee) {
        "assert" | "assert.ok" => is_true(file, args.first()),
        "assert.equal" | "assert.strictEqual" => same_literal(file, &args),
        _ => {
            let Some(expect) = callee
                .child_by_field_name("object")
                .filter(|o| o.kind() == "call_expression")
                .filter(|o| {
                    o.child_by_field_name("function")
                        .is_some_and(|f| text(file, f) == "expect")
                })
            else {
                return false;
            };
            let actual = arguments(expect.child_by_field_name("arguments"));
            let matcher = callee.child_by_field_name("property");
            match matcher.map(|m| text(file, m)) {
                Some("toBe" | "toEqual" | "toStrictEqual") => actual
                    .first()
                    .zip(args.first())
                    .is_some_and(|(a, b)| same_literal(file, &[*a, *b])),
                Some("toBeTruthy") => args.is_empty() && is_true(file, actual.first()),
                _ => false,
            }
        }
    }
}

fn py_call(file: &SourceFile, call: Node<'_>) -> bool {
    let Some(callee) = call.child_by_field_name("function") else {
        return false;
    };
    let args = arguments(call.child_by_field_name("arguments"));
    match text(file, callee) {
        "self.assertTrue" => is_true(file, args.first()),
        "self.assertEqual" => same_literal(file, &args),
        _ => false,
    }
}

fn py_assert(file: &SourceFile, assert: Node<'_>) -> bool {
    let Some(condition) = assert.named_child(0) else {
        return false;
    };
    let is_eq = condition.kind() == "comparison_operator"
        && condition.child_count() == 3
        && condition.child(1).is_some_and(|op| op.kind() == "==");
    is_true(file, Some(&condition)) || (is_eq && same_literal(file, &arguments(Some(condition))))
}

fn rs_macro(file: &SourceFile, call: Node<'_>) -> bool {
    let name = call.child_by_field_name("macro").map(|m| text(file, m));
    let mut cursor = call.walk();
    let Some(tokens) = call
        .named_children(&mut cursor)
        .find(|c| c.kind() == "token_tree")
    else {
        return false;
    };
    let args = rs_args(tokens);
    match name {
        Some("assert") => is_true(file, args.first()),
        Some("assert_eq") => same_literal(file, &args),
        _ => false,
    }
}

/// Leading macro arguments that are a single token; an argument like `a.len()` ends the list.
fn rs_args(tokens: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = tokens.walk();
    let inner: Vec<Node<'_>> = tokens
        .children(&mut cursor)
        .filter(|n| !n.is_extra())
        .collect();
    let inner = inner.get(1..inner.len().saturating_sub(1)).unwrap_or(&[]);
    inner
        .split(|n| n.kind() == ",")
        .map_while(|group| match group {
            [single] => Some(*single),
            _ => None,
        })
        .collect()
}

fn arguments(list: Option<Node<'_>>) -> Vec<Node<'_>> {
    let Some(list) = list else {
        return Vec::new();
    };
    let mut cursor = list.walk();
    list.named_children(&mut cursor)
        .filter(|n| !n.is_extra())
        .collect()
}

fn is_true(file: &SourceFile, node: Option<&Node<'_>>) -> bool {
    // Python's `True` and TS's `true` are both a `true` node; Rust has `boolean_literal`.
    node.is_some_and(|n| {
        n.kind() == "true" || (n.kind() == "boolean_literal" && text(file, *n) == "true")
    })
}

/// The first two arguments are the same literal token.
fn same_literal(file: &SourceFile, args: &[Node<'_>]) -> bool {
    match args {
        [a, b, ..] => is_literal(*a) && is_literal(*b) && text(file, *a) == text(file, *b),
        _ => false,
    }
}

fn is_literal(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        // TS and Python
        "true" | "false" | "number" | "string" | "null" | "integer" | "float" | "none"
        // Rust
        | "boolean_literal" | "integer_literal" | "float_literal" | "string_literal"
    )
}

fn text<'a>(file: &'a SourceFile, node: Node<'_>) -> &'a str {
    &file.text[node.byte_range()]
}
