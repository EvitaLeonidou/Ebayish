//! User domain entities
//! Consolidates user-related domain logic from new_user, user_email, and username

use chrono::NaiveDate;
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

#[derive(Debug, Clone)]
pub struct Username(String);

impl Username {
    pub fn parse(s: String) -> Result<Username, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        //checks that unicode characters dont escape 256 char bound
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{s} is not a valid username."))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct NewUser {
    pub username: Username,
    pub email: UserEmail,
    pub password_hash: String,
    pub first_name: Username,
    pub last_name: Username,
    pub phone: String,
    pub date_of_birth: NaiveDate,
    pub seller_rating: Option<bigdecimal::BigDecimal>,
}

#[cfg(test)]
mod tests {
    use super::{UserEmail, Username};
    use claim::{assert_err, assert_ok};

    // UserEmail tests
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

    // Username tests
    #[test]
    fn a_256_grapheme_lons_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(Username::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(Username::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(Username::parse(name));
    }

    #[test]
    fn empty_string_is_rejected_username() {
        let name = "".to_string();
        assert_err!(Username::parse(name));
    }

    #[test]
    fn username_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(Username::parse(name));
        }
    }

    #[test]
    fn valid_name_parsed_successfully() {
        let name = "Go back markel".to_string();
        assert_ok!(Username::parse(name));
    }
}