use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Sanitizer {
    allowed_chars: Regex,
    blocked_patterns: HashSet<String>,
}

impl Sanitizer {
    pub fn new() -> Self {
        let allowed_chars = Regex::new(r"^[a-zA-Z0-9_\-\.@\s]+$").unwrap();
        let blocked_patterns = HashSet::from([
            "DROP".to_string(),
            "DELETE".to_string(),
            "--".to_string(),
            ";".to_string(),
        ]);

        Self {
            allowed_chars,
            blocked_patterns,
        }
    }

    pub fn sanitize_input(&self, input: &str) -> Result<String, String> {
        // Check for blocked patterns
        let upper_input = input.to_uppercase();
        for pattern in &self.blocked_patterns {
            if upper_input.contains(pattern) {
                return Err(format!("Input contains blocked pattern: {}", pattern));
            }
        }

        // Validate characters
        if !self.allowed_chars.is_match(input) {
            return Err("Input contains invalid characters".to_string());
        }

        Ok(input.to_string())
    }

    pub fn sanitize_identifier(&self, identifier: &str) -> Result<String, String> {
        if !self.allowed_chars.is_match(identifier) {
            return Err("Identifier contains invalid characters".to_string());
        }
        Ok(identifier.to_string())
    }
}