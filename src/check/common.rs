
pub fn contains(string: &str, patterns: &Vec<String>) -> bool {
    patterns.iter()
        .map(|p| string.contains(p))
        .fold(false, |a, b| a || b)
}

