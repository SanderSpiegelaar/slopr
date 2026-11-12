use tree_sitter::Node;

use crate::config::RuleSettings;
use crate::finding::Severity;
use crate::rules::{Check, Hit, Rule};
use crate::source::{Lang, SourceFile};

pub const RULES: &[Rule] = &[
    Rule {
        id: "error/swallowed",
        summary: "Error handler that hides the failure",
        default_severity: Severity::Error,
        langs: &[
            Lang::TypeScript,
            Lang::Tsx,
            Lang::JavaScript,
            Lang::Python,
            Lang::Rust,
        ],
        default_on: true,
        check: Check::File(swallowed),
    },
    Rule {
        id: "error/redundant-rethrow",
        summary: "Catch that only rethrows the same error",
        default_severity: Severity::Warn,
        langs: &[Lang::TypeScript, Lang::Tsx, Lang::JavaScript, Lang::Python],
        default_on: true,
        check: Check::File(redundant_rethrow),
    },
];

const TS_LOGGERS: &[&str] = &["console", "logger", "log"];
const PY_LOGGERS: &[&str] = &["logging", "logger", "log", "print"];

fn swallowed(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    let Some(lang) = file.parsed.as_ref().map(|p| p.lang) else {
        return;
    };
    for node in file.nodes() {
        let message = match (lang, node.kind()) {
            (Lang::Python, "except_clause") if py_swallows(file, node) => {
                "except block swallows the error"
            }
            (Lang::Rust, "match_arm") if rs_ignores_err(file, node) => {
                "`Err(_)` arm discards the error"
            }
            (Lang::Rust, "expression_statement") if rs_discards_ok(file, node) => {
                "`.ok()` discards the error"
            }
            (Lang::Python | Lang::Rust, _) => continue,
            (_, "catch_clause") if ts_swallows(file, node) => "catch block swallows the error",
            (_, "call_expression") if ts_empty_catch_handler(file, node) => {
                "`.catch` with an empty handler swallows the error"
            }
            _ => continue,
        };
        hits.push(Hit {
            span: file.span(node.byte_range()),
            message: message.to_string(),
        });
    }
}

fn redundant_rethrow(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    for node in file.nodes() {
        let rethrows = match node.kind() {
            "catch_clause" => ts_rethrows(file, node),
            "except_clause" => py_rethrows(file, node),
            _ => continue,
        };
        if rethrows {
            hits.push(Hit {
                span: file.span(node.byte_range()),
                message: "handler only rethrows the same error".to_string(),
            });
        }
    }
}

fn ts_swallows(file: &SourceFile, clause: Node<'_>) -> bool {
    let param = clause
        .child_by_field_name("parameter")
        .map(|p| text(file, p));
    clause
        .child_by_field_name("body")
        .is_some_and(|body| statements(body).all(|s| silent_log(file, s, TS_LOGGERS, param)))
}

fn ts_empty_catch_handler(file: &SourceFile, call: Node<'_>) -> bool {
    let is_catch = call
        .child_by_field_name("function")
        .and_then(|f| f.child_by_field_name("property"))
        .is_some_and(|p| text(file, p) == "catch");
    let Some(args) = call.child_by_field_name("arguments") else {
        return false;
    };
    let mut args = statements(args);
    let handler = args.next().filter(|_| args.next().is_none());
    is_catch
        && handler.is_some_and(|h| {
            matches!(h.kind(), "arrow_function" | "function_expression")
                && h.child_by_field_name("body").is_some_and(|b| {
                    b.kind() == "statement_block" && statements(b).next().is_none()
                })
        })
}

fn ts_rethrows(file: &SourceFile, clause: Node<'_>) -> bool {
    let Some(param) = clause.child_by_field_name("parameter") else {
        return false;
    };
    clause
        .child_by_field_name("body")
        .and_then(only_statement)
        .filter(|s| s.kind() == "throw_statement")
        .and_then(only_statement)
        .is_some_and(|thrown| {
            thrown.kind() == "identifier" && text(file, thrown) == text(file, param)
        })
}

fn py_swallows(file: &SourceFile, clause: Node<'_>) -> bool {
    let alias = py_alias(file, clause);
    py_block(clause).is_some_and(|block| {
        statements(block).all(|s| {
            s.kind() == "pass_statement"
                || only_statement(s).is_some_and(|e| e.kind() == "ellipsis")
                || silent_log(file, s, PY_LOGGERS, alias)
        })
    })
}

fn py_rethrows(file: &SourceFile, clause: Node<'_>) -> bool {
    let Some(raise) = py_block(clause)
        .and_then(only_statement)
        .filter(|s| s.kind() == "raise_statement")
    else {
        return false;
    };
    let mut raised = statements(raise);
    match (raised.next(), raised.next()) {
        (None, _) => true,
        (Some(name), None) => {
            name.kind() == "identifier"
                && py_alias(file, clause).is_some_and(|a| a == text(file, name))
        }
        _ => false,
    }
}

/// The name bound by `except E as name`, which this grammar parses as an `as_pattern` value.
fn py_alias<'a>(file: &'a SourceFile, clause: Node<'_>) -> Option<&'a str> {
    clause
        .child_by_field_name("alias")
        .or_else(|| {
            clause
                .child_by_field_name("value")
                .filter(|v| v.kind() == "as_pattern")
                .and_then(|v| v.child_by_field_name("alias"))
        })
        .map(|a| text(file, a))
}

fn py_block(clause: Node<'_>) -> Option<Node<'_>> {
    statements(clause).find(|n| n.kind() == "block")
}

fn rs_ignores_err(file: &SourceFile, arm: Node<'_>) -> bool {
    let is_err_wildcard = arm
        .child_by_field_name("pattern")
        .is_some_and(|p| compact(text(file, p)) == "Err(_)");
    is_err_wildcard
        && arm.child_by_field_name("value").is_some_and(|v| {
            v.kind() == "unit_expression" || (v.kind() == "block" && statements(v).next().is_none())
        })
}

fn rs_discards_ok(file: &SourceFile, statement: Node<'_>) -> bool {
    only_statement(statement)
        .filter(|call| call.kind() == "call_expression")
        .is_some_and(|call| {
            let ok = call
                .child_by_field_name("function")
                .filter(|f| f.kind() == "field_expression")
                .and_then(|f| f.child_by_field_name("field"))
                .is_some_and(|f| text(file, f) == "ok");
            let no_args = call
                .child_by_field_name("arguments")
                .is_some_and(|a| statements(a).next().is_none());
            ok && no_args
        })
}

/// A logging call statement that leaves out the caught error.
fn silent_log(
    file: &SourceFile,
    statement: Node<'_>,
    loggers: &[&str],
    error: Option<&str>,
) -> bool {
    let Some(call) = Some(statement)
        .filter(|s| s.kind() == "expression_statement")
        .and_then(only_statement)
        .filter(|c| matches!(c.kind(), "call_expression" | "call"))
    else {
        return false;
    };
    let Some(callee) = call.child_by_field_name("function") else {
        return false;
    };
    let Some(args) = call.child_by_field_name("arguments") else {
        return false;
    };
    let mut root = callee;
    while let Some(object) = root.child_by_field_name("object") {
        root = object;
    }
    let is_logger = root.kind() == "identifier" && loggers.contains(&text(file, root));
    let logs_error = error.is_some_and(|name| mentions(file, args, name))
        || callee
            .child_by_field_name("attribute")
            .is_some_and(|a| text(file, a) == "exception")
        || statements(args).any(|a| {
            a.kind() == "keyword_argument"
                && a.child_by_field_name("name")
                    .is_some_and(|n| text(file, n) == "exc_info")
                && a.child_by_field_name("value")
                    .is_some_and(|v| v.kind() == "true")
        });
    is_logger && !logs_error
}

fn mentions(file: &SourceFile, node: Node<'_>, name: &str) -> bool {
    (node.kind() == "identifier" && text(file, node) == name)
        || node
            .children(&mut node.walk())
            .any(|c| mentions(file, c, name))
}

/// Named children, skipping comments.
fn statements(node: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    let mut cursor = node.walk();
    let children: Vec<_> = node
        .named_children(&mut cursor)
        .filter(|n| !n.is_extra())
        .collect();
    children.into_iter()
}

fn only_statement(node: Node<'_>) -> Option<Node<'_>> {
    let mut children = statements(node);
    children.next().filter(|_| children.next().is_none())
}

fn text<'a>(file: &'a SourceFile, node: Node<'_>) -> &'a str {
    &file.text[node.byte_range()]
}

fn compact(s: &str) -> String {
    s.split_whitespace().collect()
}
