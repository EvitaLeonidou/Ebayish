use crate::configuration::SeedingSettings;
use crate::domain::{NewItem, NewUser, UserEmail, Username, hash_password};
use crate::error_handling::error_chain_fmt;
use crate::services::{ImageService, ItemService, UserService};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use secrecy::Secret;
use sqlx::{PgPool, Postgres, Transaction};
use std::path::Path;
use std::str::FromStr;
use tokio::fs;
use uuid::Uuid;

type ItemsWithImages = Vec<(String, Vec<(Vec<u8>, String)>)>;

#[derive(thiserror::Error)]
pub enum SeedingError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("User service error: {0}")]
    UserServiceError(#[from] crate::services::user_service::UserServiceError),
    #[error("Item service error: {0}")]
    ItemServiceError(#[from] crate::services::item_service::ItemServiceError),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Seeding already completed")]
    AlreadyCompleted,
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SeedingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Debug, Clone)]
struct SeedItem {
    pub item_id: String,
    pub listing_type: String,
    pub name: String,
    pub first_bid: BigDecimal,
    pub currently: BigDecimal,
    pub buy_price: Option<BigDecimal>,
    pub number_of_bids: i32,
    pub location: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub started: DateTime<Utc>,
    pub ends: DateTime<Utc>,
    pub description: String,
    pub seller_rating: BigDecimal,
    pub condition: String,
    pub shipping_cost: BigDecimal,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone)]
struct SeedItemImage {
    pub data: Vec<u8>,
    pub original_name: String,
    pub _display_order: i32,
}

pub struct SeedingService;

impl SeedingService {
    const OPERATION_NAME: &'static str = "create_sellers_and_items";

    #[tracing::instrument(name = "Load images from directory")]
    async fn load_images_from_directory(
        image_dir: &Path,
        item_name: &str,
    ) -> Result<Vec<SeedItemImage>, SeedingError> {
        if !image_dir.exists() {
            tracing::warn!("Image directory does not exist: {}", image_dir.display());
            return Ok(Vec::new());
        }

        let item_dir = image_dir.join(item_name);
        if !item_dir.exists() {
            tracing::info!("No images found for item: {}", item_name);
            return Ok(Vec::new());
        }

        let mut images = Vec::new();
        let mut entries = fs::read_dir(&item_dir)
            .await
            .context("Failed to read image directory")?;

        let mut display_order = 1;
        while let Some(entry) = entries
            .next_entry()
            .await
            .context("Failed to read directory entry")?
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let extension = path //file extension check
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp") {
                continue;
            }

            let data = fs::read(&path)
                .await
                .context(format!("Failed to read image file: {}", path.display()))?;

            images.push(SeedItemImage {
                data,
                original_name: file_name.to_string(),
                _display_order: display_order,
            });

            display_order += 1;
            if display_order > 5 {
                break; // Max 5 images per item
            }
        }

        tracing::info!("Loaded {} images for item: {}", images.len(), item_name);
        Ok(images)
    }

    #[tracing::instrument(name = "Check if seeding already completed", skip(pool))]
    async fn is_seeding_completed(pool: &PgPool) -> Result<bool, SeedingError> {
        let result = sqlx::query!(
            "SELECT id FROM seeding_status WHERE operation_name = $1",
            Self::OPERATION_NAME
        )
        .fetch_optional(pool)
        .await?;

        Ok(result.is_some())
    }

    #[tracing::instrument(name = "Mark seeding as completed", skip(transaction))]
    async fn mark_seeding_completed(
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), SeedingError> {
        sqlx::query!(
            "INSERT INTO seeding_status (operation_name) VALUES ($1)",
            Self::OPERATION_NAME
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    #[tracing::instrument(name = "Create paliatzis seller user", skip(transaction, settings))]
    async fn create_paliatzis_seller(
        transaction: &mut Transaction<'_, Postgres>,
        settings: &SeedingSettings,
    ) -> Result<Uuid, SeedingError> {
        let password_hash = hash_password(Secret::new("paliatzis123".to_string()))
            .context("Failed to hash password")?;

        let new_user = NewUser {
            username: Username::parse(settings.seller_username.clone())
                .map_err(SeedingError::ValidationError)?,
            email: UserEmail::parse(settings.seller_email.clone())
                .map_err(SeedingError::ValidationError)?,
            password_hash,
            first_name: Username::parse(settings.seller_first_name.clone())
                .map_err(SeedingError::ValidationError)?,
            last_name: Username::parse(settings.seller_last_name.clone())
                .map_err(SeedingError::ValidationError)?,
            phone: settings.seller_phone.clone(),
            date_of_birth: NaiveDate::from_ymd_opt(1980, 5, 15)
                .ok_or_else(|| anyhow::anyhow!("Invalid birth date"))?,
            seller_rating: Some(bigdecimal::BigDecimal::from(97)),
            tax_id: Some("123456789".to_string()),
            location: Some("Manolates".to_string()),
            country: Some("Greece".to_string()),
        };

        let user_id = UserService::insert_user(&new_user, transaction).await?;
        tracing::info!("Created paliatzis seller user with ID: {}", user_id);
        Ok(user_id)
    }

    #[tracing::instrument(name = "Create psaras seller user", skip(transaction))]
    async fn create_psaras_seller(
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Uuid, SeedingError> {
        let password_hash = hash_password(Secret::new("psaras123".to_string()))
            .context("Failed to hash password")?;

        let new_user = NewUser {
            username: Username::parse("o psaras".to_string())
                .map_err(SeedingError::ValidationError)?,
            email: UserEmail::parse("psaras@example.com".to_string())
                .map_err(SeedingError::ValidationError)?,
            password_hash,
            first_name: Username::parse("o".to_string()).map_err(SeedingError::ValidationError)?,
            last_name: Username::parse("psaras".to_string())
                .map_err(SeedingError::ValidationError)?,
            phone: "+306947123456".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1975, 3, 20)
                .ok_or_else(|| anyhow::anyhow!("Invalid birth date"))?,
            seller_rating: Some(bigdecimal::BigDecimal::from(95)),
            tax_id: Some("987654321".to_string()),
            location: Some("Balos".to_string()),
            country: Some("Greece".to_string()),
        };

        let user_id = UserService::insert_user(&new_user, transaction).await?;
        tracing::info!("Created psaras seller user with ID: {}", user_id);
        Ok(user_id)
    }

    #[tracing::instrument(name = "Get paliatzis seed items data")]
    fn get_paliatzis_seed_items(
        _seller_user_id: Uuid,
        seller_location: &str,
        seller_country: &str,
    ) -> Vec<SeedItem> {
        let now = Utc::now();

        let manolates_lat = 37.7850;
        let manolates_lng = 26.8250;

        let seed_items = vec![
            SeedItem {
                item_id: Uuid::new_v4().to_string(),
                listing_type: "auction".to_string(),
                name: "Nikon DSLR Φωτογραφική Μηχανή D850 Full Frame".to_string(),
                first_bid: BigDecimal::from_str("1500.00").unwrap(),
                currently: BigDecimal::from_str("1500.00").unwrap(),
                buy_price: Some(BigDecimal::from_str("2200.00").unwrap()),
                number_of_bids: 0,
                location: seller_location.to_string(),
                country: seller_country.to_string(),
                latitude: manolates_lat,
                longitude: manolates_lng,
                started: now - Duration::days(3),
                ends: now + Duration::days(4),
                description: "Ο γρήγορος αισθητήρας με φορμά FX και εξαιρετικά υψηλή ανάλυση της D850 χρησιμοποιεί τα 45,7 megapixel της ωφέλιμης ανάλυσης για τη δημιουργία αρχείων 45,4 MP με πλούσιες λεπτομέρειες. Εκτυπώστε εικόνες εξαιρετικά υψηλής ανάλυσης σε ιδιαίτερα μεγάλα φορμά. Τραβήξτε video που είναι πραγματικά πλήρους κάδρου σε ποιότητα 4K. Ο αισθητήρας CMOS με οπίσθιο φωτισμό δεν έχει οπτικό χαμηλοπερατό φίλτρο και περιλαμβάνει φακούς micro on-chip με σχεδίαση χωρίς κενά καθώς και μη ανακλαστική επίστρωση.".to_string(),
                seller_rating: BigDecimal::from(98),
                condition: "Excellent".to_string(),
                shipping_cost: BigDecimal::from_str("15.00").unwrap(),
                categories: vec!["Electronics".to_string()],
            },
            SeedItem {
                item_id: Uuid::new_v4().to_string(),
                listing_type: "auction".to_string(),
                name: "Vintage Vinyl Collection - Classic Rock".to_string(),
                first_bid: BigDecimal::from_str("25.00").unwrap(),
                currently: BigDecimal::from_str("85.00").unwrap(),
                buy_price: None,
                number_of_bids: 0,
                location: seller_location.to_string(),
                country: seller_country.to_string(),
                latitude: manolates_lat,
                longitude: manolates_lng,
                started: now - Duration::days(6),
                ends: now + Duration::hours(18),
                description: "Rare collection of vintage vinyl records including Dire Straits and Fleetwood Mac. All records in excellent condition with minimal wear. A must-have for collectors.".to_string(),
                seller_rating: BigDecimal::from(96),
                condition: "Very Good".to_string(),
                shipping_cost: BigDecimal::from_str("8.50").unwrap(),
                categories: vec!["Collectibles".to_string(), "Music".to_string()],
            },
            SeedItem {
                item_id: Uuid::new_v4().to_string(),
                listing_type: "fixed_price".to_string(),
                name: "Acoustic Guitar".to_string(),
                first_bid: BigDecimal::from_str("280.00").unwrap(),
                currently: BigDecimal::from_str("280.00").unwrap(),
                buy_price: Some(BigDecimal::from_str("280.00").unwrap()),
                number_of_bids: 0,
                location: seller_location.to_string(),
                country: seller_country.to_string(),
                latitude: manolates_lat,
                longitude: manolates_lng,
                started: now - Duration::days(2),
                ends: now + Duration::days(30),
                description: "Beautiful handcrafted acoustic guitar made by local luthier. Rich, warm sound with excellent projection. Perfect for both beginners and experienced players.".to_string(),
                seller_rating: BigDecimal::from(99),
                condition: "Like New".to_string(),
                shipping_cost: BigDecimal::from_str("25.00").unwrap(),
                categories: vec!["Musical Instruments".to_string()],
            },
            SeedItem {
                item_id: Uuid::new_v4().to_string(),
                listing_type: "fixed_price".to_string(),
                name: "Ancient Ceramic Pottery Replica".to_string(),
                first_bid: BigDecimal::from_str("89.99").unwrap(),
                currently: BigDecimal::from_str("89.99").unwrap(),
                buy_price: Some(BigDecimal::from_str("89.99").unwrap()),
                number_of_bids: 0,
                location: seller_location.to_string(),
                country: seller_country.to_string(),
                latitude: manolates_lat,
                longitude: manolates_lng,
                started: now - Duration::days(1),
                ends: now + Duration::days(30),
                description: "A creation of my wife from her pottery class".to_string(),
                seller_rating: BigDecimal::from(95),
                condition: "New".to_string(),
                shipping_cost: BigDecimal::from_str("12.50").unwrap(),
                categories: vec!["Art".to_string()],
            },
            SeedItem {
                item_id: Uuid::new_v4().to_string(),
                listing_type: "auction".to_string(),
                name: "Greek Philosophers".to_string(),
                first_bid: BigDecimal::from_str("200.00").unwrap(),
                currently: BigDecimal::from_str("320.00").unwrap(),
                buy_price: Some(BigDecimal::from_str("500.00").unwrap()),
                number_of_bids: 0,
                location: seller_location.to_string(),
                country: seller_country.to_string(),
                latitude: manolates_lat,
                longitude: manolates_lng,
                started: now - Duration::hours(12),
                ends: now + Duration::days(7),
                description: "Sovaro book, sovari timh".to_string(),
                seller_rating: BigDecimal::from(97),
                condition: "Good".to_string(),
                shipping_cost: BigDecimal::from_str("5.00").unwrap(),
                categories: vec!["Books".to_string()],
            },
        ];

        seed_items
    }

    #[tracing::instrument(name = "Get psaras seed items data")]
    fn get_psaras_seed_items(
        _seller_user_id: Uuid,
        seller_location: &str,
        seller_country: &str,
    ) -> Vec<SeedItem> {
        let now = Utc::now();

        let balos_lat = 37.6985;
        let balos_lng = 26.7483;

        let seed_items = vec![
            SeedItem {
                item_id: Uuid::new_v4().to_string(),
                listing_type: "auction".to_string(),
                name: "MVD Predator Spearfishing gun 110cm".to_string(),
                first_bid: BigDecimal::from_str("150.00").unwrap(),
                currently: BigDecimal::from_str("150.00").unwrap(),
                buy_price: Some(BigDecimal::from_str("350.00").unwrap()),
                number_of_bids: 0,
                location: seller_location.to_string(),
                country: seller_country.to_string(),
                latitude: balos_lat,
                longitude: balos_lng,
                started: now - Duration::days(2),
                ends: now + Duration::days(5),
                description: "Το MVD Carbon Predator S είναι σοβαρο εργαλείο και αντιπροσωπεύει την κορυφή της απόδοσης και της καινοτομίας στην κατηγορία Ψαροντούφεκα. Κατασκευασμένο από 100% carbon, προσφέρει απαράμιλλη αντοχή, απόδοση και ακρίβεια, χάρη στις τεχνικές κατασκευής αιχμής.".to_string(),
                seller_rating: BigDecimal::from(95),
                condition: "Very Good".to_string(),
                shipping_cost: BigDecimal::from_str("20.00").unwrap(),
                categories: vec!["Fishing".to_string()],
            },
            SeedItem {
                item_id: Uuid::new_v4().to_string(),
                listing_type: "fixed_price".to_string(),
                name: "Spearfishing Wetsuit".to_string(),
                first_bid: BigDecimal::from_str("120.00").unwrap(),
                currently: BigDecimal::from_str("120.00").unwrap(),
                buy_price: Some(BigDecimal::from_str("120.00").unwrap()),
                number_of_bids: 0,
                location: seller_location.to_string(),
                country: seller_country.to_string(),
                latitude: balos_lat,
                longitude: balos_lng,
                started: now - Duration::days(1),
                ends: now + Duration::days(30),
                description: "Komple spearfishing wetsuit.".to_string(),
                seller_rating: BigDecimal::from(95),
                condition: "Excellent".to_string(),
                shipping_cost: BigDecimal::from_str("15.00").unwrap(),
                categories: vec!["Fishing".to_string(), "Clothing".to_string()],
            },
            SeedItem {
                item_id: Uuid::new_v4().to_string(),
                listing_type: "auction".to_string(),
                name: "Shimano Dialuna 2.9m".to_string(),
                first_bid: BigDecimal::from_str("150.00").unwrap(),
                currently: BigDecimal::from_str("150.00").unwrap(),
                buy_price: Some(BigDecimal::from_str("300.00").unwrap()),
                number_of_bids: 0,
                location: seller_location.to_string(),
                country: seller_country.to_string(),
                latitude: balos_lat,
                longitude: balos_lng,
                started: now - Duration::hours(6),
                ends: now + Duration::days(6),
                description: "Lightweight carbon fiber fishing rod, 2.9m length. Perfect for sea fishing from rocks or boat. Tromero exw vgalei trela pragmata me auto.".to_string(),
                seller_rating: BigDecimal::from(95),
                condition: "Good".to_string(),
                shipping_cost: BigDecimal::from_str("12.00").unwrap(),
                categories: vec!["Fishing".to_string()],
            },
        ];

        seed_items
    }

    #[tracing::instrument(
        name = "Create paliatzis items",
        skip(transaction, seller_user_id, _image_service)
    )]
    async fn create_paliatzis_items(
        transaction: &mut Transaction<'_, Postgres>,
        seller_user_id: Uuid,
        seller_location: &str,
        seller_country: &str,
        _image_service: &ImageService,
        image_dir: Option<&Path>,
    ) -> Result<ItemsWithImages, SeedingError> {
        let seed_items =
            Self::get_paliatzis_seed_items(seller_user_id, seller_location, seller_country);

        let mut items_with_images = Vec::new();

        for seed_item in seed_items {
            let is_fixed_price = seed_item.listing_type == "fixed_price";

            let new_item = NewItem {
                item_id: seed_item.item_id.clone(),
                listing_type: seed_item.listing_type.clone(),
                name: seed_item.name.clone(),
                price: seed_item.first_bid,
                currently: if is_fixed_price {
                    None
                } else {
                    Some(seed_item.currently)
                },
                buy_price: seed_item.buy_price,
                number_of_bids: if is_fixed_price {
                    None
                } else {
                    Some(seed_item.number_of_bids)
                },
                location: Some(seed_item.location),
                country: Some(seed_item.country),
                latitude: Some(seed_item.latitude),
                longitude: Some(seed_item.longitude),
                started: seed_item.started,
                ends: if is_fixed_price {
                    None
                } else {
                    Some(seed_item.ends)
                },
                description: Some(seed_item.description),
                seller_user_id,
                seller_rating: Some(seed_item.seller_rating),
                condition: Some(seed_item.condition),
                shipping_cost: seed_item.shipping_cost,
                categories: seed_item.categories.clone(),
            };

            new_item.validate().map_err(SeedingError::ValidationError)?;

            ItemService::insert_item(&new_item, transaction).await?;
            tracing::info!("Created item: {}", seed_item.item_id);

            // Load images for this item from image directory
            if let Some(img_dir) = image_dir {
                tracing::info!("Loading images for item: {}", seed_item.name);
                match Self::load_images_from_directory(img_dir, &seed_item.name).await {
                    Ok(images) => {
                        if !images.is_empty() {
                            let image_files: Vec<(Vec<u8>, String)> = images
                                .into_iter()
                                .map(|img| (img.data, img.original_name))
                                .collect();

                            tracing::info!(
                                "Prepared {} images for upload for item: {}",
                                image_files.len(),
                                seed_item.item_id
                            );
                            items_with_images.push((seed_item.item_id.clone(), image_files));
                        } else {
                            tracing::info!("No images found for item: {}", seed_item.name);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load images for item {}: {:?}",
                            seed_item.name,
                            e
                        );
                    }
                }
            }
        }

        Ok(items_with_images)
    }

    #[tracing::instrument(
        name = "Create psaras items",
        skip(transaction, seller_user_id, _image_service)
    )]
    async fn create_psaras_items(
        transaction: &mut Transaction<'_, Postgres>,
        seller_user_id: Uuid,
        seller_location: &str,
        seller_country: &str,
        _image_service: &ImageService,
        image_dir: Option<&Path>,
    ) -> Result<ItemsWithImages, SeedingError> {
        let seed_items =
            Self::get_psaras_seed_items(seller_user_id, seller_location, seller_country);

        let mut items_with_images = Vec::new();

        for seed_item in seed_items {
            let is_fixed_price = seed_item.listing_type == "fixed_price";

            let new_item = NewItem {
                item_id: seed_item.item_id.clone(),
                listing_type: seed_item.listing_type.clone(),
                name: seed_item.name.clone(),
                price: seed_item.first_bid,
                currently: if is_fixed_price {
                    None
                } else {
                    Some(seed_item.currently)
                },
                buy_price: seed_item.buy_price,
                number_of_bids: if is_fixed_price {
                    None
                } else {
                    Some(seed_item.number_of_bids)
                },
                location: Some(seed_item.location),
                country: Some(seed_item.country),
                latitude: Some(seed_item.latitude),
                longitude: Some(seed_item.longitude),
                started: seed_item.started,
                ends: if is_fixed_price {
                    None
                } else {
                    Some(seed_item.ends)
                },
                description: Some(seed_item.description),
                seller_user_id,
                seller_rating: Some(seed_item.seller_rating),
                condition: Some(seed_item.condition),
                shipping_cost: seed_item.shipping_cost,
                categories: seed_item.categories.clone(),
            };

            new_item.validate().map_err(SeedingError::ValidationError)?;

            ItemService::insert_item(&new_item, transaction).await?;
            tracing::info!("Created item: {}", seed_item.item_id);

            if let Some(img_dir) = image_dir {
                tracing::info!("Loading images for item: {}", seed_item.name);
                match Self::load_images_from_directory(img_dir, &seed_item.name).await {
                    Ok(images) => {
                        if !images.is_empty() {
                            let image_files: Vec<(Vec<u8>, String)> = images
                                .into_iter()
                                .map(|img| (img.data, img.original_name))
                                .collect();

                            tracing::info!(
                                "Prepared {} images for upload for item: {}",
                                image_files.len(),
                                seed_item.item_id
                            );
                            items_with_images.push((seed_item.item_id.clone(), image_files));
                        } else {
                            tracing::info!("No images found for item: {}", seed_item.name);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load images for item {}: {:?}",
                            seed_item.name,
                            e
                        );
                    }
                }
            }
        }

        Ok(items_with_images)
    }

    #[tracing::instrument(name = "Seed database", skip(pool, settings, image_service))]
    pub async fn seed_database(
        pool: &PgPool,
        settings: &SeedingSettings,
        image_service: &ImageService,
    ) -> Result<(), SeedingError> {
        if Self::is_seeding_completed(pool).await? {
            tracing::info!("Database seeding already completed, skipping");
            return Err(SeedingError::AlreadyCompleted);
        }

        tracing::info!("Starting database seeding process");

        // Hardcoded path to seed images in backend directory
        let image_dir = Some(Path::new("./seed_images"));

        let mut transaction = pool.begin().await?;

        let paliatzis_user_id = Self::create_paliatzis_seller(&mut transaction, settings).await?;

        let mut all_items_with_images = Self::create_paliatzis_items(
            &mut transaction,
            paliatzis_user_id,
            "Manolates",
            "Greece",
            image_service,
            image_dir,
        )
        .await?;

        let psaras_user_id = Self::create_psaras_seller(&mut transaction).await?;

        let psaras_items_with_images = Self::create_psaras_items(
            &mut transaction,
            psaras_user_id,
            "Balos",
            "Greece",
            image_service,
            image_dir,
        )
        .await?;

        all_items_with_images.extend(psaras_items_with_images);

        Self::mark_seeding_completed(&mut transaction).await?;

        transaction.commit().await?;

        for (item_id, image_files) in all_items_with_images {
            if !image_files.is_empty() {
                tracing::info!(
                    "Uploading {} images for item: {}",
                    image_files.len(),
                    item_id
                );

                match image_service
                    .upload_images(&item_id, image_files, pool)
                    .await
                {
                    Ok(uploaded_images) => {
                        tracing::info!(
                            "Successfully uploaded {} images for item: {}",
                            uploaded_images.len(),
                            item_id
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to upload images for item {}: {:?}", item_id, e);
                    }
                }
            }
        }

        tracing::info!("Database seeding completed successfully");
        Ok(())
    }
}
