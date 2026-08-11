//! Category-related DTOs

use serde::{Deserialize, Serialize};
use teddy_domain::{CategoryName, NewCategory};

#[derive(Debug, Deserialize)]
pub struct CategoryRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

impl TryFrom<CategoryRequest> for NewCategory {
    type Error = String;

    fn try_from(value: CategoryRequest) -> Result<NewCategory, String> {
        let name = CategoryName::parse(value.name)?;
        Ok(Self { name })
    }
}