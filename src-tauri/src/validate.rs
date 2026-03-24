use crate::config::Config;

pub fn is_valid_selection(text: &str, config: &Config) -> bool {
    let trimmed = text.trim();

    if trimmed.len() < config.min_length {
        return false;
    }

    if config.english_only && !contains_english(trimmed) {
        return false;
    }

    true
}

fn contains_english(text: &str) -> bool {
    text.chars().any(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_valid_english_text() {
        assert!(is_valid_selection("Hello world", &default_config()));
    }

    #[test]
    fn test_too_short() {
        assert!(!is_valid_selection("Hi", &default_config()));
    }

    #[test]
    fn test_empty_string() {
        assert!(!is_valid_selection("", &default_config()));
    }

    #[test]
    fn test_whitespace_only() {
        assert!(!is_valid_selection("   ", &default_config()));
    }

    #[test]
    fn test_numbers_only() {
        let config = default_config();
        assert!(!is_valid_selection("12345", &config));
    }

    #[test]
    fn test_japanese_only_with_english_only_on() {
        let config = default_config();
        assert!(!is_valid_selection("こんにちは世界", &config));
    }

    #[test]
    fn test_japanese_only_with_english_only_off() {
        let config = Config {
            english_only: false,
            ..default_config()
        };
        assert!(is_valid_selection("こんにちは世界", &config));
    }

    #[test]
    fn test_mixed_text() {
        assert!(is_valid_selection("Hello こんにちは", &default_config()));
    }

    #[test]
    fn test_exactly_min_length() {
        let config = Config {
            min_length: 4,
            ..default_config()
        };
        assert!(is_valid_selection("test", &config));
    }

    #[test]
    fn test_trimmed_below_min_length() {
        assert!(!is_valid_selection("  ab  ", &default_config()));
    }

    #[test]
    fn test_custom_min_length() {
        let config = Config {
            min_length: 10,
            ..default_config()
        };
        assert!(!is_valid_selection("short", &config));
        assert!(is_valid_selection("long enough text", &config));
    }
}
