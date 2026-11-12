use tree_sitter::Node;

use crate::config::RuleSettings;
use crate::finding::Severity;
use crate::rules::{Check, Hit, Rule};
use crate::source::{Lang, SourceFile};

pub const RULES: &[Rule] = &[Rule {
    id: "debug/leftover",
    summary: "Debug output or breakpoint left in library code",
    default_severity: Severity::Warn,
    langs: &[
        Lang::TypeScript,
        Lang::Tsx,
        Lang::JavaScript,
        Lang::Python,
        Lang::Rust,
    ],
    default_on: true,
    check: Check::File(leftover),
}];

const CONSOLE_METHODS: &[&str] = &["log", "debug", "info", "trace", "dir"];

fn leftover(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    let Some(parsed) = &file.parsed else { return };
    if file
        .path
        .split('/')
        .rev()
        .skip(1)
        .any(|d| d == "bin" || d == "scripts")
    {
        return;
    }
    let text = |n: Node<'_>| &file.text[n.byte_range()];
    let allow_print = match parsed.lang {
        Lang::TypeScript | Lang::Tsx | Lang::JavaScript if file.text.starts_with("#!") => return,
        Lang::Python => {
            file.path.rsplit('/').next() == Some("__main__.py")
                || has_main_guard(file, parsed.tree.root_node())
        }
        _ => false,
    };
    for node in file.nodes() {
        let found = match (parsed.lang, node.kind()) {
            (Lang::Rust, "macro_invocation") => node
                .child_by_field_name("macro")
                .map(text)
                .filter(|m| *m == "dbg" || m.ends_with("::dbg"))
                .map(|_| "dbg!"),
            (Lang::Python, "call") => node.child_by_field_name("function").map(text).filter(|f| {
                matches!(*f, "breakpoint" | "pdb.set_trace") || (*f == "print" && !allow_print)
            }),
            (Lang::Python, "import_statement") => {
                let mut cursor = node.walk();
                node.children_by_field_name("name", &mut cursor)
                    .map(|n| n.child_by_field_name("name").unwrap_or(n))
                    .any(|n| text(n) == "pdb")
                    .then_some("import pdb")
            }
            (Lang::Python | Lang::Rust, _) => None,
            (_, "debugger_statement") => Some("debugger"),
            (_, "call_expression") => node
                .child_by_field_name("function")
                .filter(|f| f.kind() == "member_expression")
                .filter(|f| f.child_by_field_name("object").map(text) == Some("console"))
                .filter(|f| {
                    f.child_by_field_name("property")
                        .is_some_and(|p| CONSOLE_METHODS.contains(&text(p)))
                })
                .map(text),
            _ => None,
        };
        if let Some(what) = found.filter(|_| !file.in_test(node)) {
            hits.push(Hit {
                span: file.span(node.byte_range()),
                message: format!("`{what}` left in library code"),
            });
        }
    }
}

/// A top-level `if __name__ == "__main__":` block marks a script entry point.
fn has_main_guard(file: &SourceFile, module: Node<'_>) -> bool {
    let mut cursor = module.walk();
    module
        .named_children(&mut cursor)
        .filter(|n| n.kind() == "if_statement")
        .filter_map(|n| n.child_by_field_name("condition"))
        .any(|c| {
            let condition: String = file.text[c.byte_range()]
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .map(|ch| if ch == '\'' { '"' } else { ch })
                .collect();
            condition == "__name__==\"__main__\"" || condition == "\"__main__\"==__name__"
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hits(path: &str, is_test: bool) -> usize {
        let file = SourceFile::new(
            path.to_string(),
            "console.log(1);\n".to_string(),
            Some(Lang::TypeScript),
            is_test,
        );
        let mut hits = Vec::new();
        leftover(&file, &RuleSettings::default(), &mut hits);
        hits.len()
    }

    #[test]
    fn skips_bin_scripts_and_test_files() {
        assert_eq!(hits("src/lib.ts", false), 1);
        assert_eq!(hits("bin.ts", false), 1);
        assert_eq!(hits("bin/cli.ts", false), 0);
        assert_eq!(hits("tools/scripts/gen.ts", false), 0);
        assert_eq!(hits("src/lib.test.ts", true), 0);
    }
}
