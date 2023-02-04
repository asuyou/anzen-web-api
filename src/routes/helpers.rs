use regex::Regex;
use super::errors::RegexError;

async fn validate_password(password: &str) -> Result<(), RegexError>
{
    // Lookahead is not allowed so we must use custom statements
    // "(?=.{10,})(?=.+[a-z])(?=.+[A-Z])(?=.+\d)(?=.+[!@"£$%^&*#-])(?!.\s)"
    //
    // Original regex meanings
    // (?=.{10,}) - Lookahead for at least 10 characters
    // (?=.+[a-z]) - Lookahead for a lower case character
    // (?=.+[A-Z]) - Lookahead for a upper case character
    // (?=.+\n) - Lookahed for at least 1 digit
    // (?=.+[!@"£$%^&*#-]) - Lookahead for a special character
    // (?!.\s) - Ensures there is no whitespace

    let validators = [
        // Lowercase Characters
        Regex::new("[a-z]").unwrap(),
        // Upercase Characters
        Regex::new("[A-Z]").unwrap(),
        // digit regex
        Regex::new(r#"\d"#).unwrap(),
        // Special characters
        Regex::new(r#"[!"£$%^&*\[\];'#~?><\\]"#).unwrap(),
        // Length is 16 or longer
        Regex::new(".{16,}").unwrap(),
        // No whitespace
        Regex::new(r#"^\S+$"#).unwrap()
    ];

    match validators.iter().all(|validator| validator.is_match(password)) {
        true => Ok(()),
        false => Err(RegexError::new("Password does not meet strength criteria"))
    }
}

async fn validate_email(email: &str) -> Result<(), RegexError> {

    // Regex modified from https://emailregex.com/
    let validator = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();

    match validator.is_match(email) {
        true => Ok(()),
        false => Err(RegexError::new("Invalid email address"))
    }
}

