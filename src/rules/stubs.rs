use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::Node;

use crate::config::RuleSettings;
use crate::finding::Severity;
use crate::rules::{Check, Hit, Rule};
use crate::source::{Lang, SourceFile};

pub const RULES: &[Rule] = &[Rule {
    id: "stub/placeholder-body",
    summary: "Function body left as a placeholder",
    default_severity: Severity::Warn,
    langs: &[
        Lang::TypeScript,
        Lang::Tsx,
        Lang::JavaScript,
        Lang::Python,
        Lang::Rust,
    ],
    default_on: true,
    check: Check::File(placeholder_body),
}];

static TS_MESSAGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)not implemented|implement (me|this)|todo").expect("valid regex")
});

fn placeholder_body(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    let Some(lang) = file.parsed.as_ref().map(|p| p.lang) else {
        return;
    };
    if lang == Lang::Python && file.path.ends_with(".pyi") {
        return;
    }
    for node in file.nodes() {
        let stub = match lang {
            Lang::TypeScript | Lang::Tsx | Lang::JavaScript => ts_stub(file, node),
            Lang::Python => py_stub(file, node),
            Lang::Rust => rs_stub(file, node),
        };
        if let Some(stub) = stub {
            hits.push(Hit {
                span: file.span(stub.byte_range()),
                message: "function body is only a placeholder".to_string(),
            });
        }
    }
}

fn ts_stub<'a>(file: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    if !matches!(
        node.kind(),
        "function_declaration"
            | "function_expression"
            | "generator_function_declaration"
            | "generator_function"
            | "arrow_function"
            | "method_definition"
    ) {
        return None;
    }
    let statement = only_statement(node.child_by_field_name("body")?, false)?;
    let new = statement.named_child(0)?;
    let constructor = new.child_by_field_name("constructor")?;
    let message = new.child_by_field_name("arguments")?.named_child(0)?;
    let is_stub = statement.kind() == "throw_statement"
        && new.kind() == "new_expression"
        && &file.text[constructor.byte_range()] == "Error"
        && matches!(message.kind(), "string" | "template_string")
        && TS_MESSAGE.is_match(&file.text[message.byte_range()]);
    is_stub.then_some(statement)
}

fn py_stub<'a>(file: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    if node.kind() != "function_definition" || py_exempt(file, node) {
        return None;
    }
    let statement = only_statement(node.child_by_field_name("body")?, true)?;
    let text = |n: Node<'_>| &file.text[n.byte_range()];
    let is_stub = match statement.kind() {
        "pass_statement" => true,
        "expression_statement" => statement
            .named_child(0)
            .is_some_and(|e| e.kind() == "ellipsis"),
        "raise_statement" => statement.named_child(0).is_some_and(|e| {
            let error = match e.kind() {
                "call" => e.child_by_field_name("function"),
                _ => Some(e),
            };
            error.is_some_and(|n| text(n) == "NotImplementedError")
        }),
        _ => false,
    };
    is_stub.then_some(statement)
}

/// Decorated `abstractmethod` or `overload`, or a method of a `Protocol`, `ABC` or `ABCMeta` class.
fn py_exempt(file: &SourceFile, function: Node<'_>) -> bool {
    let text = |n: Node<'_>| &file.text[n.byte_range()];
    let outer = function
        .parent()
        .filter(|p| p.kind() == "decorated_definition");
    if let Some(decorated) = outer {
        let mut cursor = decorated.walk();
        let mut names = decorated
            .named_children(&mut cursor)
            .filter(|d| d.kind() == "decorator")
            .filter_map(|d| d.named_child(0))
            .map(|e| text(e).rsplit('.').next().unwrap_or_default());
        if names.any(|name| matches!(name, "abstractmethod" | "overload")) {
            return true;
        }
    }
    let class = outer
        .unwrap_or(function)
        .parent()
        .filter(|b| b.kind() == "block")
        .and_then(|b| b.parent())
        .filter(|c| c.kind() == "class_definition");
    class
        .and_then(|c| c.child_by_field_name("superclasses"))
        .is_some_and(|bases| {
            text(bases)
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|t| matches!(t, "Protocol" | "ABC" | "ABCMeta"))
        })
}

fn rs_stub<'a>(file: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    if node.kind() != "function_item"
        || std::iter::successors(node.parent(), Node::parent).any(|n| n.kind() == "trait_item")
    {
        return None;
    }
    let statement = only_statement(node.child_by_field_name("body")?, false)?;
    let call = match statement.kind() {
        "expression_statement" => statement.named_child(0)?,
        _ => statement,
    };
    let name = call.child_by_field_name("macro")?;
    let name = file.text[name.byte_range()].rsplit("::").next()?;
    (call.kind() == "macro_invocation" && matches!(name, "todo" | "unimplemented"))
        .then_some(statement)
}

/// The body's one statement, ignoring comments and, when asked, a leading docstring.
fn only_statement(body: Node<'_>, skip_docstring: bool) -> Option<Node<'_>> {
    let mut cursor = body.walk();
    let mut statements = body
        .named_children(&mut cursor)
        .filter(|n| !n.kind().contains("comment"))
        .peekable();
    let is_docstring = |n: &Node<'_>| {
        n.kind() == "expression_statement"
            && n.named_child_count() == 1
            && n.named_child(0).is_some_and(|s| s.kind() == "string")
    };
    if skip_docstring {
        statements.next_if(is_docstring);
    }
    let only = statements.next()?;
    statements.next().is_none().then_some(only)
}
