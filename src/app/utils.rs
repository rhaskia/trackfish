use ndarray::Array1;

/// If two strings are functionally the same, eg barring capitalisation, punctuation, whitespace
pub fn similar(str1: &str, str2: &str) -> bool {
    strip_unnessecary(str1) == strip_unnessecary(str2)
}

/// Removes whitespace, punctuation, and sets to lowercase so album and artist names that differ
/// slightly cannot cause issues with duplications
pub fn strip_unnessecary(s: &str) -> String {
    s.chars()
        .filter(|c| !(c.is_whitespace() || c.is_ascii_punctuation()))
        .collect::<String>()
        .to_lowercase()
}

/// Linear interpolation implementation
pub fn lerp(a: &Array1<f32>, b: &Array1<f32>, t: f32) -> Array1<f32> {
    (1.0 - t) * a + t * b
}

/// Returns a string with every word's first letter capitalised
pub fn title_case(s: &str) -> String {
    let mut result = String::new();
    let mut last_whitespace = true;

    for c in s.chars() {
        if last_whitespace {
            result.push_str(&c.to_uppercase().collect::<String>());
        } else {
            result.push(c);
        }
        last_whitespace = c.is_whitespace();
    }

    result
}
