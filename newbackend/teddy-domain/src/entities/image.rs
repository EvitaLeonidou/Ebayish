use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemImage {
    pub id: Uuid,
    pub item_id: String,
    pub filename: String,
    pub original_name: String,
    pub display_order: i32,
    pub file_size: i64,
    pub mime_type: String,
    pub upload_timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewItemImage {
    pub item_id: String,
    pub filename: String,
    pub original_name: String,
    pub display_order: i32,
    pub file_size: i64,
    pub mime_type: String,
}

impl NewItemImage {
    pub fn validate(&self) -> Result<(), String> {
        // Validate item_id
        if self.item_id.trim().is_empty() {
            return Err("Item ID cannot be empty".to_string());
        }

        // Validate filename
        if self.filename.trim().is_empty() {
            return Err("Filename cannot be empty".to_string());
        }

        // Validate original_name
        if self.original_name.trim().is_empty() {
            return Err("Original filename cannot be empty".to_string());
        }

        if self.original_name.len() > 255 {
            return Err("Original filename cannot exceed 255 characters".to_string());
        }

        // Validate display_order
        if !(1..=5).contains(&self.display_order) {
            return Err("Display order must be between 1 and 5".to_string());
        }

        // Validate file_size
        if self.file_size <= 0 {
            return Err("File size must be greater than zero".to_string());
        }

        const MAX_FILE_SIZE: i64 = 10 * 1024 * 1024; // 10MB
        if self.file_size > MAX_FILE_SIZE {
            return Err("File size cannot exceed 10MB".to_string());
        }

        // Validate mime_type
        if !Self::is_valid_mime_type(&self.mime_type) {
            return Err("Invalid file type. Only JPEG, PNG, and WebP are allowed".to_string());
        }

        Ok(())
    }

    fn is_valid_mime_type(mime_type: &str) -> bool {
        matches!(mime_type, "image/jpeg" | "image/png" | "image/webp")
    }
}

pub struct FileValidator;

impl FileValidator {
    /// Validate file type by checking magic bytes (file signature)
    pub fn validate_image_type(file_data: &[u8]) -> Result<String, String> {
        if file_data.len() < 12 {
            return Err("File too small to validate".to_string());
        }

        // Check JPEG
        if file_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok("image/jpeg".to_string());
        }

        // Check PNG
        if file_data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Ok("image/png".to_string());
        }

        // Check WebP
        if file_data.len() >= 12 && file_data[0..4] == [0x52, 0x49, 0x46, 0x46]  // RIFF
            && file_data[8..12] == [0x57, 0x45, 0x42, 0x50]
        // WEBP
        {
            return Ok("image/webp".to_string());
        }

        Err("Unsupported image format".to_string())
    }

    /// Get file extension from MIME type
    pub fn get_extension_from_mime(mime_type: &str) -> Result<&'static str, String> {
        match mime_type {
            "image/jpeg" => Ok("jpg"),
            "image/png" => Ok("png"),
            "image/webp" => Ok("webp"),
            _ => Err("Unsupported MIME type".to_string()),
        }
    }
}

pub struct DisplayOrderManager;

impl DisplayOrderManager {
    /// Get the next available display order for an item
    pub fn next_available_order(existing_orders: &[i32]) -> Result<i32, String> {
        if existing_orders.len() >= 5 {
            return Err("Item already has maximum number of images (5)".to_string());
        }

        for order in 1..=5 {
            if !existing_orders.contains(&order) {
                return Ok(order);
            }
        }

        Err("No available display order slots".to_string())
    }

    /// Validate that display orders are unique and within bounds
    pub fn validate_orders(orders: &[i32]) -> Result<(), String> {
        if orders.len() > 5 {
            return Err("Cannot have more than 5 images per item".to_string());
        }

        let mut unique_orders = std::collections::HashSet::new();
        for &order in orders {
            if !(1..=5).contains(&order) {
                return Err("Display order must be between 1 and 5".to_string());
            }
            if !unique_orders.insert(order) {
                return Err("Duplicate display order found".to_string());
            }
        }

        Ok(())
    }
}