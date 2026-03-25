pub fn is_fuzzy_subsequence(query: &str, target: &str) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return true;
    }

    let mut query_chars = query.chars().map(|c| c.to_ascii_lowercase());
    let mut current = query_chars.next();

    for target_char in target.chars().map(|c| c.to_ascii_lowercase()) {
        if let Some(expected) = current {
            if target_char == expected {
                current = query_chars.next();
                if current.is_none() {
                    return true;
                }
            }
        } else {
            return true;
        }
    }

    current.is_none()
}
