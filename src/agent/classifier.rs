use crate::config::schema::QueryClassificationConfig;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassificationDecision {
    pub hint: String,
    pub priority: i32,
}

/// Classify a user message against the configured rules and return the
/// matching hint string, if any.
///
/// Returns `None` when classification is disabled, no rules are configured,
/// or no rule matches the message.
pub fn classify(config: &QueryClassificationConfig, message: &str) -> Option<String> {
    classify_with_decision(config, message).map(|decision| decision.hint)
}

/// Classify a user message and return the matched hint together with
/// match metadata for observability.
pub fn classify_with_decision(
    config: &QueryClassificationConfig,
    message: &str,
) -> Option<ClassificationDecision> {
    if !config.enabled || config.rules.is_empty() {
        return None;
    }

    let lower = message.to_lowercase();
    let len = message.len();

    let mut rules: Vec<_> = config.rules.iter().collect();
    rules.sort_by(|a, b| b.priority.cmp(&a.priority));

    for rule in rules {
        // Length constraints
        if let Some(min) = rule.min_length {
            if len < min {
                continue;
            }
        }
        if let Some(max) = rule.max_length {
            if len > max {
                continue;
            }
        }

        // Check keywords (case-insensitive) and patterns (case-sensitive)
        let keyword_hit = rule
            .keywords
            .iter()
            .any(|kw: &String| lower.contains(&kw.to_lowercase()));
        let pattern_hit = rule
            .patterns
            .iter()
            .any(|pat: &String| message.contains(pat.as_str()));
        let regex_hit = rule.regex_patterns.iter().any(|pat| {
            Regex::new(&format!("(?i){pat}"))
                .map(|re| re.is_match(message))
                .unwrap_or(false)
        });

        if keyword_hit || pattern_hit || regex_hit {
            return Some(ClassificationDecision {
                hint: rule.hint.clone(),
                priority: rule.priority,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{ClassificationRule, QueryClassificationConfig};

    fn make_config(enabled: bool, rules: Vec<ClassificationRule>) -> QueryClassificationConfig {
        QueryClassificationConfig { enabled, rules }
    }

    #[test]
    fn disabled_returns_none() {
        let config = make_config(
            false,
            vec![ClassificationRule {
                hint: "fast".into(),
                keywords: vec!["hello".into()],
                ..Default::default()
            }],
        );
        assert_eq!(classify(&config, "hello"), None);
    }

    #[test]
    fn empty_rules_returns_none() {
        let config = make_config(true, vec![]);
        assert_eq!(classify(&config, "hello"), None);
    }

    #[test]
    fn keyword_match_case_insensitive() {
        let config = make_config(
            true,
            vec![ClassificationRule {
                hint: "fast".into(),
                keywords: vec!["hello".into()],
                ..Default::default()
            }],
        );
        assert_eq!(classify(&config, "HELLO world"), Some("fast".into()));
    }

    #[test]
    fn pattern_match_case_sensitive() {
        let config = make_config(
            true,
            vec![ClassificationRule {
                hint: "code".into(),
                patterns: vec!["fn ".into()],
                ..Default::default()
            }],
        );
        assert_eq!(classify(&config, "fn main()"), Some("code".into()));
        assert_eq!(classify(&config, "FN MAIN()"), None);
    }

    #[test]
    fn length_constraints() {
        let config = make_config(
            true,
            vec![ClassificationRule {
                hint: "fast".into(),
                keywords: vec!["hi".into()],
                max_length: Some(10),
                ..Default::default()
            }],
        );
        assert_eq!(classify(&config, "hi"), Some("fast".into()));
        assert_eq!(
            classify(&config, "hi there, how are you doing today?"),
            None
        );

        let config2 = make_config(
            true,
            vec![ClassificationRule {
                hint: "reasoning".into(),
                keywords: vec!["explain".into()],
                min_length: Some(20),
                ..Default::default()
            }],
        );
        assert_eq!(classify(&config2, "explain"), None);
        assert_eq!(
            classify(&config2, "explain how this works in detail"),
            Some("reasoning".into())
        );
    }

    #[test]
    fn priority_ordering() {
        let config = make_config(
            true,
            vec![
                ClassificationRule {
                    hint: "fast".into(),
                    keywords: vec!["code".into()],
                    priority: 1,
                    ..Default::default()
                },
                ClassificationRule {
                    hint: "code".into(),
                    keywords: vec!["code".into()],
                    priority: 10,
                    ..Default::default()
                },
            ],
        );
        assert_eq!(classify(&config, "write some code"), Some("code".into()));
    }

    #[test]
    fn no_match_returns_none() {
        let config = make_config(
            true,
            vec![ClassificationRule {
                hint: "fast".into(),
                keywords: vec!["hello".into()],
                ..Default::default()
            }],
        );
        assert_eq!(classify(&config, "something completely different"), None);
    }

    #[test]
    fn regex_pattern_match() {
        let config = make_config(
            true,
            vec![ClassificationRule {
                hint: "heavy".into(),
                regex_patterns: vec![r"\d+개.*(만들|생성)".into()],
                ..Default::default()
            }],
        );
        assert_eq!(
            classify(&config, "버튼 3개 만들어줘"),
            Some("heavy".into())
        );
        assert_eq!(classify(&config, "날씨 어때"), None);
    }

    #[test]
    fn regex_pattern_case_insensitive() {
        let config = make_config(
            true,
            vec![ClassificationRule {
                hint: "code".into(),
                regex_patterns: vec![r"python|javascript|rust".into()],
                ..Default::default()
            }],
        );
        assert_eq!(
            classify(&config, "Python으로 서버 만들어"),
            Some("code".into())
        );
        assert_eq!(
            classify(&config, "RUST로 코딩해줘"),
            Some("code".into())
        );
    }

    #[test]
    fn invalid_regex_does_not_panic() {
        let config = make_config(
            true,
            vec![ClassificationRule {
                hint: "bad".into(),
                regex_patterns: vec![r"[invalid".into()],
                ..Default::default()
            }],
        );
        assert_eq!(classify(&config, "anything"), None);
    }

    #[test]
    fn classify_with_decision_exposes_priority_of_matched_rule() {
        let config = make_config(
            true,
            vec![
                ClassificationRule {
                    hint: "fast".into(),
                    keywords: vec!["code".into()],
                    priority: 3,
                    ..Default::default()
                },
                ClassificationRule {
                    hint: "code".into(),
                    keywords: vec!["code".into()],
                    priority: 10,
                    ..Default::default()
                },
            ],
        );

        let decision = classify_with_decision(&config, "write code now")
            .expect("classification decision expected");
        assert_eq!(decision.hint, "code");
        assert_eq!(decision.priority, 10);
    }
}
