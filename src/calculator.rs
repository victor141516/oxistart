use meval;

/// Check if a string looks like a mathematical expression
pub fn is_math_expression(text: &str) -> bool {
    if text.trim().is_empty() {
        return false;
    }

    // Check if the string contains mathematical operators
    let has_operator = text.contains('+')
        || text.contains('-')
        || text.contains('*')
        || text.contains('/')
        || text.contains('^')
        || text.contains('(')
        || text.contains(')');

    // Check if it contains at least one digit
    let has_digit = text.chars().any(|c| c.is_ascii_digit());

    has_operator && has_digit
}

/// Evaluate a mathematical expression and return the result as a string
pub fn evaluate(expression: &str) -> Option<String> {
    match meval::eval_str(expression) {
        Ok(result) => Some(format!("{}", result)),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_math_expression() {
        assert!(is_math_expression("2+2"));
        assert!(is_math_expression("10 * 5"));
        assert!(is_math_expression("(3+4)*2"));
        assert!(!is_math_expression("hello"));
        assert!(!is_math_expression(""));
        assert!(!is_math_expression("123")); // Just a number, no operator
    }

    #[test]
    fn test_evaluate_simple() {
        assert_eq!(evaluate("2+2"), Some("4".to_string()));
        assert_eq!(evaluate("10*5"), Some("50".to_string()));
        assert_eq!(evaluate("100/4"), Some("25".to_string()));
    }

    #[test]
    fn test_evaluate_complex() {
        assert_eq!(evaluate("(3+4)*2"), Some("14".to_string()));
        assert_eq!(evaluate("2^3"), Some("8".to_string()));
    }

    #[test]
    fn test_evaluate_invalid() {
        assert_eq!(evaluate("abc"), None);
        assert_eq!(evaluate("2///2"), None);
        assert_eq!(evaluate("(("), None);
    }
}
