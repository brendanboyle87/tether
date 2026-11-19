//! Glob matching for rule patterns.
//!
//! `*` matches any run of characters within one path segment, `?` one such
//! character, and `**` any run including `/`. A `**/` also matches nothing, so
//! `**/*.rs` covers `a.rs` as well as `src/a.rs`. Everything else is literal.

pub fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    matches(&pattern, &text)
}

fn matches(pattern: &[char], text: &[char]) -> bool {
    match pattern {
        [] => text.is_empty(),
        ['*', '*', rest @ ..] => {
            let zero_dirs = matches!(rest, ['/', ..]) && matches(&rest[1..], text);
            zero_dirs || (0..=text.len()).any(|skip| matches(rest, &text[skip..]))
        }
        ['*', rest @ ..] => {
            let segment = text.iter().position(|c| *c == '/').unwrap_or(text.len());
            (0..=segment).any(|skip| matches(rest, &text[skip..]))
        }
        ['?', rest @ ..] => match text {
            [c, tail @ ..] if *c != '/' => matches(rest, tail),
            _ => false,
        },
        [literal, rest @ ..] => match text {
            [c, tail @ ..] if c == literal => matches(rest, tail),
            _ => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::glob_match;

    #[test]
    fn literals_and_single_stars() {
        assert!(glob_match("sandbox.exec", "sandbox.exec"));
        assert!(!glob_match("sandbox.exec", "sandbox.exec2"));
        assert!(glob_match("net.*", "net.fetch"));
        assert!(glob_match("net.*", "net."));
        assert!(!glob_match("net.*", "network.fetch"));
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "src/main.rs"));
        assert!(glob_match("src/*/mod.rs", "src/a/mod.rs"));
        assert!(!glob_match("src/*/mod.rs", "src/a/b/mod.rs"));
        assert!(glob_match("a?c", "abc"));
        assert!(!glob_match("a?c", "a/c"));
    }

    #[test]
    fn double_star_crosses_directories() {
        assert!(glob_match("src/**", "src/a.rs"));
        assert!(glob_match("src/**", "src/a/b/c.rs"));
        assert!(!glob_match("src/**", "src"));
        assert!(!glob_match("src/**", "srcs/a.rs"));
        assert!(glob_match("**/*.rs", "a.rs"));
        assert!(glob_match("**/*.rs", "src/deep/a.rs"));
        assert!(!glob_match("**/*.rs", "src/a.txt"));
        assert!(glob_match("**", ""));
        assert!(glob_match("**", "anything/at/all"));
    }
}
