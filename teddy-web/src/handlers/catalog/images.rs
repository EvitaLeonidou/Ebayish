//! src/handlers/catalog/images.rs

use teddy_domain::ItemImage;
use crate::Claims;
use teddy_services::ImageService;
use crate::errors::catalog::ImageError;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use futures_util::TryStreamExt;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;


#[derive(Serialize)]
pub struct ImageUploadResponse {
    images: Vec<ItemImageResponse>,
}

#[derive(Serialize, Clone)]
pub struct ItemImageResponse {
    id: String,
    filename: String,
    original_name: String,
    display_order: i32,
    url: String,
}

impl From<ItemImage> for ItemImageResponse {
    fn from(image: ItemImage) -> Self {
        Self {
            id: image.id.to_string(),
            filename: image.filename.clone(),
            original_name: image.original_name,
            display_order: image.display_order,
            url: format!("/uploads/items/{}/{}", image.item_id, image.filename),
        }
    }
}

#[tracing::instrument(
    name = "Upload item images",
    skip(multipart, pool, image_service, claims)
)]
pub async fn upload_images(
    path: web::Path<String>,
    mut multipart: Multipart,
    pool: web::Data<PgPool>,
    image_service: web::Data<ImageService>,
    claims: Claims,
) -> Result<HttpResponse, ImageError> {
    let item_id = path.into_inner();

    // Verify item exists and user has permission
    verify_item_access(&item_id, &pool, &claims).await?;

    let mut files = Vec::new();
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
    const MAX_FILES: usize = 5;

    // Process multipart form data
    while let Some(mut field) = multipart
        .try_next()
        .await
        .map_err(|_| ImageError::InvalidMultipartData)?
    {
        if files.len() >= MAX_FILES {
            return Err(ImageError::TooManyImages);
        }

        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .get_filename()
            .unwrap_or("unknown")
            .to_string();

        if filename.is_empty() || filename == "unknown" {
            continue; // Skip fields without filenames
        }

        // Read file data
        let mut file_data = Vec::new();
        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(|_| ImageError::InvalidMultipartData)?
        {
            file_data.extend_from_slice(&chunk);

            if file_data.len() > MAX_FILE_SIZE {
                return Err(ImageError::FileTooLarge);
            }
        }

        if !file_data.is_empty() {
            files.push((file_data, filename));
        }
    }

    if files.is_empty() {
        return Err(ImageError::InvalidMultipartData);
    }

    // Upload files using ImageService
    let uploaded_images = image_service
        .upload_images(&item_id, files, pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("Image upload failed: {:?}", e);
            match e.to_string().as_str() {
                s if s.contains("exceeds maximum size") => ImageError::FileTooLarge,
                s if s.contains("Invalid file type") => ImageError::InvalidFileType,
                s if s.contains("Cannot upload") && s.contains("max:") => ImageError::TooManyImages,
                _ => ImageError::StorageError,
            }
        })?;

    let response_images: Vec<ItemImageResponse> = uploaded_images
        .into_iter()
        .map(ItemImageResponse::from)
        .collect();

    Ok(HttpResponse::Created().json(ImageUploadResponse {
        images: response_images,
    }))
}

#[derive(Serialize)]
pub struct ItemImagesResponse {
    images: Vec<ItemImageResponse>,
}

#[tracing::instrument(name = "Get item images", skip(pool, image_service))]
pub async fn get_item_images(
    path: web::Path<String>,
    pool: web::Data<PgPool>,
    image_service: web::Data<ImageService>,
) -> Result<HttpResponse, ImageError> {
    let item_id = path.into_inner();

    // Verify item exists (no auth required for viewing)
    verify_item_exists(&item_id, &pool).await?;

    let images = image_service
        .get_item_images(&item_id, pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get item images: {:?}", e);
            ImageError::StorageError
        })?;

    let response_images: Vec<ItemImageResponse> =
        images.into_iter().map(ItemImageResponse::from).collect();

    Ok(HttpResponse::Ok().json(ItemImagesResponse {
        images: response_images,
    }))
}

#[derive(Serialize)]
pub struct DeleteImageResponse {
    success: bool,
}

#[tracing::instrument(name = "Delete image", skip(pool, image_service, claims))]
pub async fn delete_image(
    path: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    image_service: web::Data<ImageService>,
    claims: Claims,
) -> Result<HttpResponse, ImageError> {
    let (item_id, image_id_str) = path.into_inner();

    let image_id = Uuid::parse_str(&image_id_str).map_err(|_| ImageError::ImageNotFound)?;

    // Verify item exists and user has permission
    verify_item_access(&item_id, &pool, &claims).await?;

    let success = image_service
        .delete_image(&item_id, image_id, pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete image: {:?}", e);
            ImageError::StorageError
        })?;

    if !success {
        return Err(ImageError::ImageNotFound);
    }

    Ok(HttpResponse::Ok().json(DeleteImageResponse { success: true }))
}

#[tracing::instrument(name = "Serve image file", skip(image_service))]
pub async fn serve_image(
    path: web::Path<(String, String)>,
    image_service: web::Data<ImageService>,
) -> Result<NamedFile, ImageError> {
    let (item_id, filename) = path.into_inner();

    // Validate filename to prevent path traversal
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(ImageError::ImageNotFound);
    }

    let file_path = image_service.get_image_path(&item_id, &filename);

    // Check if file exists
    if !image_service.image_exists(&item_id, &filename).await {
        return Err(ImageError::ImageNotFound);
    }

    NamedFile::open(file_path).map_err(|e| {
        tracing::error!("Failed to serve image file: {:?}", e);
        ImageError::ImageNotFound
    })
}

/// Verify that the item exists and user has permission to modify it
async fn verify_item_access(
    item_id: &str,
    pool: &PgPool,
    claims: &Claims,
) -> Result<(), ImageError> {
    // Get item and verify ownership
    let item = sqlx::query!(
        r#"SELECT seller_user_id FROM items WHERE item_id = $1"#,
        item_id
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch item for authorization")
    .map_err(|_| ImageError::ItemNotFound)?
    .ok_or(ImageError::ItemNotFound)?;

    // Check if user is owner or admin
    let seller_id = item.seller_user_id.ok_or(ImageError::ItemNotFound)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| ImageError::UnauthorizedAccess)?;
    let is_admin = claims.role == "admin";

    if user_id != seller_id && !is_admin {
        return Err(ImageError::UnauthorizedAccess);
    }

    Ok(())
}

/// Verify that the item exists (for public endpoints)
async fn verify_item_exists(item_id: &str, pool: &PgPool) -> Result<(), ImageError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM items WHERE item_id = $1)"#,
        item_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to check item existence")
    .map_err(|_| ImageError::ItemNotFound)?
    .unwrap_or(false);

    if !exists {
        return Err(ImageError::ItemNotFound);
    }

    Ok(())
}