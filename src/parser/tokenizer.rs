pub fn tokenize_segment(segment: &str) -> Vec<Vec<&str>> {
    segment
        .trim_end_matches('\'')
        .split('+')
        .map(|part| part.split(':').collect())
        .collect()
}