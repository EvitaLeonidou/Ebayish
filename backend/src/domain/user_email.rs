//! src/domain/user_email.rs

use unicode_segmentation::UnicodeSegmentation;
use validator::validate_email;

#[derive(Debug, Clone)]
pub struct UserEmail(String);

impl UserEmail {
    pub fn parse(s: String) -> Result<UserEmail, String> {
        let is_not_too_long = s.graphemes(true).count() <= 256;
        if validate_email(&s) && is_not_too_long {
            Ok(Self(s))
        } else {
            Err(format!("{s} is not a valid user email."))
        }
    }
}

impl AsRef<str> for UserEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::UserEmail;
    use claim::assert_err;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(UserEmail::parse(email));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "deathwish.com".to_string();
        assert_err!(UserEmail::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@deathwish.com".to_string();
        assert_err!(UserEmail::parse(email));
    }
}
