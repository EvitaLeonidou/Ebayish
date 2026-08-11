use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct CategoryName(String);

impl CategoryName {
    pub fn parse(s: String) -> Result<CategoryName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 100;

        if is_empty_or_whitespace || is_too_long {
            Err(format!("{s} is not a valid category name."))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for CategoryName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct NewCategory {
    pub name: CategoryName,
}

#[cfg(test)]
mod tests {
    use crate::domain::CategoryName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_100_grapheme_long_name_is_valid() {
        let name = "a".repeat(100);
        assert_ok!(CategoryName::parse(name));
    }

    #[test]
    fn a_name_longer_than_100_graphemes_is_rejected() {
        let name = "a".repeat(101);
        assert_err!(CategoryName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(CategoryName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(CategoryName::parse(name));
    }

    #[test]
    fn valid_name_parsed_successfully() {
        let name = "Electronics".to_string();
        assert_ok!(CategoryName::parse(name));
    }
}
