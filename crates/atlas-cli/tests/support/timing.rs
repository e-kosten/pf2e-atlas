#![allow(dead_code)]

pub fn timing_value<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.lines()
        .find(|line| line.starts_with("timing: "))
        .and_then(|line| {
            line.split_whitespace()
                .find_map(|part| part.strip_prefix(prefix))
        })
        .expect("expected timing value")
}

pub fn value_after_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.lines()
        .find_map(|line| line.strip_prefix(prefix))
        .expect("expected line with prefix")
}

pub fn assert_human_duration(value: &str) {
    assert!(
        value.ends_with("ms")
            || value.ends_with('s')
            || value.contains("m ")
            || value.contains("h "),
        "expected human-readable duration, got {value:?}"
    );
}
