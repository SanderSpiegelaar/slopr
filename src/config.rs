use regex::Regex;

pub struct RuleSettings {
    #[allow(dead_code, reason = "read by comment/untracked-todo")]
    pub todo_issue_pattern: Regex,
}

impl Default for RuleSettings {
    fn default() -> Self {
        RuleSettings {
            todo_issue_pattern: Regex::new(r"(#\d+|[A-Z]+-\d+|https?://)")
                .expect("default issue pattern is a valid regex"),
        }
    }
}
