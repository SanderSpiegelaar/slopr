use tree_sitter::Node;

use crate::config::RuleSettings;
use crate::finding::Severity;
use crate::rules::{Check, Hit, Rule};
use crate::source::{Lang, SourceFile};

pub const RULES: &[Rule] = &[Rule {
    id: "ts/unsafe-assertion",
    summary: "Type assertion to any, or a double assertion through unknown",
    default_severity: Severity::Warn,
    langs: &[Lang::TypeScript, Lang::Tsx],
    default_on: true,
    check: Check::File(unsafe_assertion),
}];

fn unsafe_assertion(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    let is_type = |node: Option<Node<'_>>, name: &str| {
        node.is_some_and(|n| n.kind() == "predefined_type" && &file.text[n.byte_range()] == name)
    };
    for node in file.nodes() {
        let message = match node.kind() {
            "as_expression" if is_type(node.named_child(1), "any") => "`as any` assertion",
            "as_expression" if as_unknown(node.named_child(0), is_type) => {
                "`as unknown as` double assertion"
            }
            "type_assertion"
                if is_type(node.named_child(0).and_then(|t| t.named_child(0)), "any") =>
            {
                "`<any>` assertion"
            }
            _ => continue,
        };
        hits.push(Hit {
            span: file.span(node.byte_range()),
            message: message.to_string(),
        });
    }
}

fn as_unknown(inner: Option<Node<'_>>, is_type: impl Fn(Option<Node<'_>>, &str) -> bool) -> bool {
    inner.is_some_and(|n| n.kind() == "as_expression" && is_type(n.named_child(1), "unknown"))
}
