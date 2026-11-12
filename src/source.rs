use std::ops::Range;

use tree_sitter::{Language, Node, Parser, Tree};

use crate::finding::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang {
    TypeScript,
    Tsx,
    JavaScript,
    Python,
    Rust,
}

impl Lang {
    fn grammar(self) -> Language {
        match self {
            Lang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            // No JS grammar crate is in the dependency list, and TSX parses JS.
            Lang::Tsx | Lang::JavaScript => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Lang::Python => tree_sitter_python::LANGUAGE.into(),
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
        }
    }
}

pub struct Parsed {
    pub lang: Lang,
    pub tree: Tree,
}

pub struct SourceFile {
    pub path: String,
    pub text: String,
    pub parsed: Option<Parsed>,
    pub is_test: bool,
    line_starts: Vec<usize>,
}

impl SourceFile {
    /// Parses `text` when `lang` is set. A tree with ERROR nodes is kept.
    pub fn new(path: String, text: String, lang: Option<Lang>, is_test: bool) -> Self {
        let parsed = lang.map(|lang| {
            let mut parser = Parser::new();
            parser
                .set_language(&lang.grammar())
                .expect("grammar ABI matches the tree-sitter crate");
            let tree = parser
                .parse(&text, None)
                .expect("parse without a timeout or cancel flag returns a tree");
            Parsed { lang, tree }
        });
        let line_starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        SourceFile {
            path,
            text,
            parsed,
            is_test,
            line_starts,
        }
    }

    pub fn span(&self, bytes: Range<usize>) -> Span {
        let (start_line, start_col) = self.position(bytes.start);
        let (end_line, end_col) = self.position(bytes.end);
        Span {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    fn position(&self, byte: usize) -> (u32, u32) {
        let line = self.line_starts.partition_point(|&start| start <= byte) - 1;
        let col = self.text[self.line_starts[line]..byte].chars().count() + 1;
        (to_u32(line + 1), to_u32(col))
    }

    /// Every node in preorder, skipping ERROR subtrees. Empty for unparsed files.
    pub fn nodes(&self) -> impl Iterator<Item = Node<'_>> {
        let mut stack: Vec<Node<'_>> = self.parsed.iter().map(|p| p.tree.root_node()).collect();
        std::iter::from_fn(move || {
            loop {
                let node = stack.pop()?;
                if node.is_error() {
                    continue;
                }
                let first_child = stack.len();
                stack.extend(node.children(&mut node.walk()));
                stack[first_child..].reverse();
                return Some(node);
            }
        })
    }

    /// `(span, text with markers stripped, is_doc)` per comment node.
    #[allow(
        dead_code,
        reason = "read by the comment rule tickets, which cannot touch this file"
    )]
    pub fn comments(&self) -> impl Iterator<Item = (Span, &str, bool)> {
        self.nodes()
            .filter(|n| matches!(n.kind(), "comment" | "line_comment" | "block_comment"))
            .map(|n| {
                let raw = self.text[n.byte_range()].trim_end();
                let (text, is_doc) = strip_comment(raw);
                let start = n.start_byte();
                (self.span(start..start + raw.len()), text, is_doc)
            })
    }

    /// `(span, unquoted text, is_docstring)` per string node.
    #[allow(
        dead_code,
        reason = "read by the string rule tickets, which cannot touch this file"
    )]
    pub fn strings(&self) -> impl Iterator<Item = (Span, &str, bool)> {
        self.nodes()
            .filter(|n| {
                matches!(
                    n.kind(),
                    "string" | "template_string" | "string_literal" | "raw_string_literal"
                )
            })
            .map(|n| {
                (
                    self.span(n.byte_range()),
                    &self.text[unquoted(n)],
                    is_docstring(n),
                )
            })
    }

    /// True in a test file, and for Rust inside a `#[cfg(test)]` mod or a `#[test]` fn.
    #[allow(
        dead_code,
        reason = "read by the test rule tickets, which cannot touch this file"
    )]
    pub fn in_test(&self, node: Node<'_>) -> bool {
        if self.is_test {
            return true;
        }
        if !matches!(
            self.parsed,
            Some(Parsed {
                lang: Lang::Rust,
                ..
            })
        ) {
            return false;
        }
        std::iter::successors(Some(node), Node::parent).any(|n| match n.kind() {
            "mod_item" => self.has_attribute(n, "#[cfg(test)]"),
            "function_item" => self.has_attribute(n, "#[test]"),
            _ => false,
        })
    }

    /// Rust attributes are siblings before the item, possibly with comments between.
    fn has_attribute(&self, item: Node<'_>, attribute: &str) -> bool {
        std::iter::successors(item.prev_named_sibling(), Node::prev_named_sibling)
            .take_while(|n| {
                matches!(
                    n.kind(),
                    "attribute_item" | "line_comment" | "block_comment"
                )
            })
            .any(|n| {
                let text: String = self.text[n.byte_range()].split_whitespace().collect();
                text == attribute
            })
    }
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).expect("files over 1 MiB are skipped, so positions fit in u32")
}

fn strip_comment(raw: &str) -> (&str, bool) {
    if let Some(inner) = raw.strip_prefix("/*") {
        let inner = inner.strip_suffix("*/").unwrap_or(inner);
        let is_doc = inner.starts_with('!')
            || (inner.starts_with('*') && !inner.starts_with("**") && inner.len() > 1);
        let inner = if is_doc { &inner[1..] } else { inner };
        (inner.trim(), is_doc)
    } else if let Some(inner) = raw.strip_prefix("//") {
        let is_doc = inner.starts_with('!') || (inner.starts_with('/') && !inner.starts_with("//"));
        let inner = if is_doc { &inner[1..] } else { inner };
        (inner.trim(), is_doc)
    } else {
        (raw.strip_prefix('#').unwrap_or(raw).trim(), false)
    }
}

/// Byte range between the opening and closing delimiter tokens.
fn unquoted(node: Node<'_>) -> Range<usize> {
    if node.kind() == "raw_string_literal" {
        let mut cursor = node.walk();
        let content = node
            .named_children(&mut cursor)
            .find(|c| c.kind() == "string_content");
        return content.map_or(node.end_byte()..node.end_byte(), |c| c.byte_range());
    }
    let last = node.child_count().saturating_sub(1);
    match (node.child(0), node.child(last)) {
        (Some(open), Some(close)) if last > 0 => open.end_byte()..close.start_byte(),
        _ => node.byte_range(),
    }
}

/// A Python string that is the first statement of a module, class or function body.
fn is_docstring(node: Node<'_>) -> bool {
    let Some(statement) = node
        .parent()
        .filter(|p| p.kind() == "expression_statement" && p.named_child_count() == 1)
    else {
        return false;
    };
    let Some(body) = statement.parent() else {
        return false;
    };
    let is_body = match body.kind() {
        "module" => true,
        "block" => body
            .parent()
            .is_some_and(|p| matches!(p.kind(), "function_definition" | "class_definition")),
        _ => false,
    };
    let mut cursor = body.walk();
    let first = body
        .named_children(&mut cursor)
        .find(|c| c.kind() != "comment");
    is_body && first == Some(statement)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, lang: Lang, text: &str) -> SourceFile {
        SourceFile::new(path.to_string(), text.to_string(), Some(lang), false)
    }

    #[test]
    fn span_counts_unicode_scalars() {
        let f = SourceFile::new("a".into(), "x\né = 1".into(), None, false);
        let start = f.text.find('=').expect("has =");
        let span = f.span(start..start + 1);
        assert_eq!(
            (span.start_line, span.start_col, span.end_line, span.end_col),
            (2, 3, 2, 4)
        );
    }

    #[test]
    fn comments_strip_markers_and_flag_docs() {
        let f = file(
            "a.rs",
            Lang::Rust,
            "/// doc\n// plain\n/* block */\n//! inner\nfn f() {}\n",
        );
        let got: Vec<_> = f
            .comments()
            .map(|(s, t, d)| (s.start_line, s.end_line, t, d))
            .collect();
        assert_eq!(
            got,
            [
                (1, 1, "doc", true),
                (2, 2, "plain", false),
                (3, 3, "block", false),
                (4, 4, "inner", true)
            ]
        );
        let f = file("a.ts", Lang::TypeScript, "/** doc */\n// x\n/**/\n");
        let got: Vec<_> = f.comments().map(|(_, t, d)| (t, d)).collect();
        assert_eq!(got, [("doc", true), ("x", false), ("", false)]);
        let f = file("a.py", Lang::Python, "# hi\n");
        assert_eq!(
            f.comments().map(|(_, t, d)| (t, d)).collect::<Vec<_>>(),
            [("hi", false)]
        );
    }

    #[test]
    fn strings_unquote_and_flag_docstrings() {
        let f = file(
            "a.py",
            Lang::Python,
            "\"\"\"mod\"\"\"\ndef f():\n    'doc'\n    x = f\"a{b}\"\n",
        );
        let got: Vec<_> = f.strings().map(|(_, t, d)| (t, d)).collect();
        assert_eq!(got, [("mod", true), ("doc", true), ("a{b}", false)]);
        let f = file(
            "a.rs",
            Lang::Rust,
            "fn f() { let a = \"x\"; let b = r#\"y\"#; let c = b\"z\"; }",
        );
        let got: Vec<_> = f.strings().map(|(_, t, _)| t).collect();
        assert_eq!(got, ["x", "y", "z"]);
        let f = file(
            "a.ts",
            Lang::TypeScript,
            "const a = 'x'; const b = `y${z}`; const c = \"\";",
        );
        let got: Vec<_> = f.strings().map(|(_, t, _)| t).collect();
        assert_eq!(got, ["x", "y${z}", ""]);
    }

    #[test]
    fn in_test_finds_rust_test_ancestors() {
        let text = "fn lib() { 1; }\n#[cfg(test)]\nmod tests {\n    fn helper() { 2; }\n}\n#[test]\nfn t() { 3; }\n";
        let f = file("a.rs", Lang::Rust, text);
        let in_test = |needle: &str| {
            let at = f.text.find(needle).expect("needle present");
            let node = f
                .nodes()
                .filter(|n| n.start_byte() == at)
                .last()
                .expect("node at needle");
            f.in_test(node)
        };
        assert_eq!(
            (in_test("1;"), in_test("2;"), in_test("3;")),
            (false, true, true)
        );
    }
}
