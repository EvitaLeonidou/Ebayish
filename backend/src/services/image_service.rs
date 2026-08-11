use crate::configuration::{Settings, UploadSettings};
use crate::domain::{FileValidator, ItemImage, NewItemImage};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

#[derive(Debug)]
pub struct ImageService {
    upload_dir: PathBuf,
    max_file_size: usize,
    max_files_per_item: usize,
    allowed_types: Vec<String>,
    temp_dir: PathBuf,
}

impl ImageService {
    pub fn new(config: &UploadSettings) -> Self {
        Self {
            upload_dir: config.get_upload_path(),
            max_file_size: config.max_file_size,
            max_files_per_item: config.max_files_per_item,
            allowed_types: config.allowed_types.clone(),
            temp_dir: config.get_temp_path(),
        }
    }

    pub fn from_settings(settings: &Settings) -> Self {
        Self::new(&settings.uploads)
    }

    //keep compatibility with old implementation
    pub fn new_with_path(upload_dir: impl AsRef<Path>) -> Self {
        let default_config = UploadSettings::default();
        Self {
            upload_dir: upload_dir.as_ref().to_path_buf(),
            max_file_size: default_config.max_file_size,
            max_files_per_item: default_config.max_files_per_item,
            allowed_types: default_config.allowed_types,
            temp_dir: upload_dir.as_ref().join("temp"),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        if !self.upload_dir.exists() {
            fs::create_dir_all(&self.upload_dir)
                .await
                .context("Failed to create upload directory")?;
        }

        if !self.temp_dir.exists() {
            fs::create_dir_all(&self.temp_dir)
                .await
                .context("Failed to create temp directory")?;
        }

        Ok(())
    }

    pub async fn upload_images(
        &self,
        item_id: &str,
        files: Vec<(Vec<u8>, String)>, //file_data, original_name
        pool: &PgPool,
    ) -> Result<Vec<ItemImage>> {
        let current_count = self.get_image_count(item_id, pool).await?;

        if current_count + files.len() > self.max_files_per_item {
            anyhow::bail!(
                "Cannot upload {} files. Item already has {} images (max: {})",
                files.len(),
                current_count,
                self.max_files_per_item
            );
        }

        let mut uploaded_images = Vec::new();
        let mut transaction = pool.begin().await?;

        let mut existing_orders = self
            .get_existing_display_orders(item_id, &mut transaction)
            .await?;

        for (index, (file_data, original_name)) in files.into_iter().enumerate() {
            if file_data.len() > self.max_file_size {
                anyhow::bail!(
                    "File '{}' exceeds maximum size of {} bytes",
                    original_name,
                    self.max_file_size
                );
            }

            let mime_type = FileValidator::validate_image_type(&file_data)
                .map_err(|e| anyhow::anyhow!("Invalid file type for '{}': {}", original_name, e))?;

            if !self.allowed_types.contains(&mime_type) {
                anyhow::bail!(
                    "File type '{}' not allowed for file '{}'",
                    mime_type,
                    original_name
                );
            }

            let extension = FileValidator::get_extension_from_mime(&mime_type)
                .map_err(|e| anyhow::anyhow!(e))?;
            let filename = format!("{}.{}", Uuid::new_v4(), extension);

            let display_order =
                self.get_next_display_order(&existing_orders, current_count + index)?;

            existing_orders.push(display_order);

            let item_dir = self.upload_dir.join("items").join(item_id);
            if !item_dir.exists() {
                fs::create_dir_all(&item_dir)
                    .await
                    .context("Failed to create item directory")?;
            }

            let file_path = item_dir.join(&filename);
            let mut file = fs::File::create(&file_path)
                .await
                .context("Failed to create image file")?;
            file.write_all(&file_data)
                .await
                .context("Failed to write image data")?;

            let new_image = NewItemImage {
                item_id: item_id.to_string(),
                filename: filename.clone(),
                original_name,
                display_order,
                file_size: file_data.len() as i64,
                mime_type,
            };

            new_image
                .validate()
                .map_err(|e| anyhow::anyhow!("Validation error: {}", e))?;

            let image = self
                .insert_image_metadata(&new_image, &mut transaction)
                .await?;
            uploaded_images.push(image);
        }

        transaction
            .commit()
            .await
            .context("Failed to commit image upload transaction")?;

        Ok(uploaded_images)
    }

    fn get_next_display_order(
        &self,
        existing_orders: &[i32],
        _current_count: usize,
    ) -> Result<i32> {
        for order in 1..=5 {
            if !existing_orders.contains(&order) {
                return Ok(order);
            }
        }
        anyhow::bail!("No available display order slots");
    }

    async fn insert_image_metadata(
        &self,
        new_image: &NewItemImage,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<ItemImage> {
        let image_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query!(
            r#"INSERT INTO item_images (id, item_id, filename, original_name, display_order, file_size, mime_type, upload_timestamp)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            image_id,
            new_image.item_id,
            new_image.filename,
            new_image.original_name,
            new_image.display_order,
            new_image.file_size,
            new_image.mime_type,
            now.naive_utc()
        )
        .execute(&mut **transaction)
        .await
        .context("Failed to insert image metadata")?;

        Ok(ItemImage {
            id: image_id,
            item_id: new_image.item_id.clone(),
            filename: new_image.filename.clone(),
            original_name: new_image.original_name.clone(),
            display_order: new_image.display_order,
            file_size: new_image.file_size,
            mime_type: new_image.mime_type.clone(),
            upload_timestamp: now,
        })
    }

    pub async fn get_item_images(&self, item_id: &str, pool: &PgPool) -> Result<Vec<ItemImage>> {
        let rows = sqlx::query(
            r#"SELECT id, item_id, filename, original_name, display_order, file_size, mime_type, upload_timestamp
               FROM item_images 
               WHERE item_id = $1 
               ORDER BY display_order"#,
        )
        .bind(item_id)
        .fetch_all(pool)
        .await
        .context("Failed to fetch item images")?;

        let mut images = Vec::new();
        for row in rows {
            let image = ItemImage {
                id: row.try_get("id")?,
                item_id: row.try_get("item_id")?,
                filename: row.try_get("filename")?,
                original_name: row.try_get("original_name")?,
                display_order: row.try_get("display_order")?,
                file_size: row.try_get("file_size")?,
                mime_type: row.try_get("mime_type")?,
                upload_timestamp: {
                    let ts: chrono::NaiveDateTime = row.try_get("upload_timestamp")?;
                    DateTime::from_naive_utc_and_offset(ts, Utc)
                },
            };
            images.push(image);
        }

        Ok(images)
    }

    async fn get_image_count(&self, item_id: &str, pool: &PgPool) -> Result<usize> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM item_images WHERE item_id = $1"#,
            item_id
        )
        .fetch_one(pool)
        .await
        .context("Failed to get image count")?
        .unwrap_or(0);

        Ok(count as usize)
    }

    async fn get_existing_display_orders(
        &self,
        item_id: &str,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<i32>> {
        let orders = sqlx::query_scalar!(
            r#"SELECT display_order FROM item_images WHERE item_id = $1"#,
            item_id
        )
        .fetch_all(&mut **transaction)
        .await
        .context("Failed to get existing display orders")?;

        Ok(orders)
    }

    pub async fn delete_image(&self, item_id: &str, image_id: Uuid, pool: &PgPool) -> Result<bool> {
        let mut transaction = pool.begin().await?;

        let image = sqlx::query!(
            r#"SELECT filename FROM item_images WHERE id = $1 AND item_id = $2"#,
            image_id,
            item_id
        )
        .fetch_optional(&mut *transaction)
        .await
        .context("Failed to fetch image for deletion")?;

        let Some(image_row) = image else {
            return Ok(false);
        };

        sqlx::query!(
            r#"DELETE FROM item_images WHERE id = $1 AND item_id = $2"#,
            image_id,
            item_id
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to delete image from database")?;

        let file_path = self
            .upload_dir
            .join("items")
            .join(item_id)
            .join(&image_row.filename);

        #[allow(clippy::collapsible_if)]
        if file_path.exists() {
            if let Err(e) = fs::remove_file(&file_path).await {
                // Log error but don't fail the transaction
                tracing::warn!("Failed to delete image file {}: {}", file_path.display(), e);
            }
        }

        transaction
            .commit()
            .await
            .context("Failed to commit image deletion")?;

        Ok(true)
    }

    pub async fn delete_all_item_images(&self, item_id: &str, pool: &PgPool) -> Result<()> {
        let _images = self.get_item_images(item_id, pool).await?;

        let mut transaction = pool.begin().await?;

        sqlx::query!(r#"DELETE FROM item_images WHERE item_id = $1"#, item_id)
            .execute(&mut *transaction)
            .await
            .context("Failed to delete images from database")?;

        transaction
            .commit()
            .await
            .context("Failed to commit bulk image deletion")?;

        let item_dir = self.upload_dir.join("items").join(item_id);

        #[allow(clippy::collapsible_if)]
        if item_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&item_dir).await {
                tracing::warn!(
                    "Failed to delete item image directory {}: {}",
                    item_dir.display(),
                    e
                );
            }
        }

        Ok(())
    }

    pub fn get_image_path(&self, item_id: &str, filename: &str) -> PathBuf {
        self.upload_dir.join("items").join(item_id).join(filename)
    }

    pub async fn image_exists(&self, item_id: &str, filename: &str) -> bool {
        let path = self.get_image_path(item_id, filename);
        path.exists()
    }
}
