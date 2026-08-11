use serde::Serialize;
use crate::handlers::catalog::images::ItemImageResponse;

#[derive(Serialize)]
pub struct ImageUploadResponse {
    pub images: Vec<ItemImageResponse>,
}

#[derive(Serialize)]
pub struct ItemImagesResponse {
    pub images: Vec<ItemImageResponse>,
}

#[derive(Serialize)]
pub struct DeleteImageResponse {
    pub success: bool,
}