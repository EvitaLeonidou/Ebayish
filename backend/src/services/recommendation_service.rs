#![allow(clippy::needless_borrow)]
#![allow(clippy::type_complexity)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::needless_range_loop)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RecommendationService {
    model: Option<MatrixFactorizationModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatrixFactorizationModel {
    user_factors: Vec<Vec<f32>>,
    item_factors: Vec<Vec<f32>>,
    user_map: HashMap<String, usize>, //uuid for user and items as strings
    item_map: HashMap<String, usize>,
    latent_factors: usize,
    trained_at: DateTime<Utc>,
}

#[derive(Debug)]
struct InteractionData {
    user_id: Uuid,
    item_id: String,
    weight: f32,
}

const MODEL_PATH: &str = "/app/models/recommendation_model.json";
const LATENT_FACTORS: usize = 50;
const LEARNING_RATE: f32 = 0.01;
const REGULARIZATION: f32 = 0.001;
const ITERATIONS: usize = 100;

impl RecommendationService {
    pub fn new() -> Self {
        let model = Self::load_model().ok();
        Self { model }
    }

    pub async fn train_model(
        &mut self,
        db_pool: &PgPool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting recommendation model training");

        let interactions = self.fetch_interactions(db_pool).await?;
        tracing::info!("Fetched {} interactions for training", interactions.len());

        if interactions.is_empty() {
            tracing::warn!("No interactions found, skipping model training");
            return Ok(());
        }

        let (matrix, user_map, item_map) = self.build_interaction_matrix(interactions);
        let (user_factors, item_factors) = self.matrix_factorization(&matrix);

        // uuids  to strings for serialization
        let user_map_str: HashMap<String, usize> = user_map
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        let model = MatrixFactorizationModel {
            user_factors,
            item_factors,
            user_map: user_map_str,
            item_map,
            latent_factors: LATENT_FACTORS,
            trained_at: Utc::now(),
        };

        self.model = Some(model.clone());
        self.save_model(&model)?;

        tracing::info!("Model training completed and saved");
        Ok(())
    }

    pub async fn get_recommendations(
        &self,
        user_id: &Uuid,
        limit: usize,
        db_pool: &PgPool,
    ) -> Vec<String> {
        let model = match &self.model {
            Some(m) => m,
            None => {
                tracing::warn!("No trained model available for recommendations");
                return Vec::new();
            }
        };

        let user_idx = match model.user_map.get(&user_id.to_string()) {
            Some(&idx) => idx,
            None => {
                tracing::debug!("User {} not found in trained model", user_id);
                return Vec::new();
            }
        };

        let mut item_scores: Vec<(String, f32)> = model
            .item_map
            .iter()
            .map(|(item_id, &item_idx)| {
                let score = self.predict_rating(&model, user_idx, item_idx);
                (item_id.clone(), score)
            })
            .collect();

        item_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        //ensures that sold items are not recommended and that the user own items are not
        //recommended
        let mut active_recommendations = Vec::new();
        for (item_id, _score) in item_scores {
            if active_recommendations.len() >= limit {
                break;
            }

         match (
            self.is_item_active(db_pool, &item_id).await,
            self.is_user_own_item(db_pool, user_id, &item_id).await
        ) {
            (Ok(true), Ok(false)) => {
                active_recommendations.push(item_id);
            }
            _ => {} //skip
        }        

        }
        active_recommendations
    }

    async fn is_item_active(&self, db_pool: &PgPool, item_id: &str) -> Result<bool, sqlx::Error> {
        let row = sqlx::query!("SELECT status FROM items WHERE item_id = $1", item_id)
            .fetch_optional(db_pool)
            .await?;

        match row {
            Some(item) => {
                let status = item.status.unwrap_or_else(|| "active".to_string());
                Ok(status != "sold" && status != "ended")
            }
            None => Ok(false),
        }
    }

    async fn is_user_own_item(
        &self,
        db_pool: &PgPool,
        user_id: &Uuid,
        item_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT seller_user_id FROM items WHERE item_id = $1",
            item_id
        )
        .fetch_optional(db_pool)
        .await?;

        match row {
            Some(item) => Ok(item.seller_user_id == Some(*user_id)),
            None => Ok(false),
        }
    }

    async fn fetch_interactions(
        &self,
        db_pool: &PgPool,
    ) -> Result<Vec<InteractionData>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            WITH interaction_data AS (
                -- Purchases (weight 5.0)
                SELECT p.buyer_user_id as user_id, p.item_id, 5.0 as weight
                FROM purchases p

                UNION ALL

                -- Bids (weight 3.0)
                SELECT b.bidder_user_id as user_id, b.item_id, 3.0 as weight
                FROM bids b

                UNION ALL

                -- Category views (weight 1.0)
                SELECT ucv.user_id, i.item_id, 1.0 as weight
                FROM user_category_views ucv
                JOIN item_categories ic ON ic.category_id = ucv.category_id
                JOIN items i ON i.item_id = ic.item_id
                WHERE i.status NOT IN ('sold', 'ended')
            )
            SELECT user_id, item_id, MAX(weight) as weight
            FROM interaction_data
            GROUP BY user_id, item_id
            ORDER BY user_id, item_id
            "#
        )
        .fetch_all(db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                //skip missing data
                let user_id = row.user_id?;
                let item_id = row.item_id?;
                let weight = row.weight?.to_string().parse::<f32>().ok()?;

                Some(InteractionData {
                    user_id,
                    item_id,
                    weight,
                })
            })
            .collect())
    }

    fn build_interaction_matrix(
        &self,
        interactions: Vec<InteractionData>,
    ) -> (Vec<Vec<f32>>, HashMap<Uuid, usize>, HashMap<String, usize>) {
        let mut user_map = HashMap::new();
        let mut item_map = HashMap::new();

        for interaction in &interactions {
            if !user_map.contains_key(&interaction.user_id) {
                user_map.insert(interaction.user_id, user_map.len());
            }
            if !item_map.contains_key(&interaction.item_id) {
                item_map.insert(interaction.item_id.clone(), item_map.len());
            }
        }

        let num_users = user_map.len();
        let num_items = item_map.len();
        let mut matrix = vec![vec![0.0; num_items]; num_users];

        for interaction in interactions {
            let user_idx = user_map[&interaction.user_id];
            let item_idx = item_map[&interaction.item_id];
            matrix[user_idx][item_idx] = interaction.weight;
        }

        (matrix, user_map, item_map)
    }

    fn matrix_factorization(&self, matrix: &Vec<Vec<f32>>) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
        let num_users = matrix.len();
        let num_items = if num_users > 0 { matrix[0].len() } else { 0 };

        if num_users == 0 || num_items == 0 {
            return (Vec::new(), Vec::new());
        }

        let mut user_factors = vec![vec![0.0; LATENT_FACTORS]; num_users];
        let mut item_factors = vec![vec![0.0; LATENT_FACTORS]; num_items];

        for i in 0..num_users {
            for f in 0..LATENT_FACTORS {
                user_factors[i][f] = (rand::random::<f32>() - 0.5) * 0.1;
            }
        }
        for j in 0..num_items {
            for f in 0..LATENT_FACTORS {
                item_factors[j][f] = (rand::random::<f32>() - 0.5) * 0.1;
            }
        }

        for iteration in 0..ITERATIONS {
            let mut total_error = 0.0;
            let mut count = 0;

            for i in 0..num_users {
                for j in 0..num_items {
                    if matrix[i][j] > 0.0 {
                        let prediction = self.dot_product(&user_factors[i], &item_factors[j]);
                        let error = matrix[i][j] - prediction;
                        total_error += error * error;
                        count += 1;

                        for f in 0..LATENT_FACTORS {
                            let user_feature = user_factors[i][f];
                            let item_feature = item_factors[j][f];

                            user_factors[i][f] += LEARNING_RATE
                                * (error * item_feature - REGULARIZATION * user_feature);
                            item_factors[j][f] += LEARNING_RATE
                                * (error * user_feature - REGULARIZATION * item_feature);
                        }
                    }
                }
            }

            if iteration % 20 == 0 && count > 0 {
                let rmse = (total_error / count as f32).sqrt();
                tracing::debug!("Iteration {}: RMSE = {:.4}", iteration, rmse);
            }
        }

        (user_factors, item_factors)
    }

    fn predict_rating(
        &self,
        model: &MatrixFactorizationModel,
        user_idx: usize,
        item_idx: usize,
    ) -> f32 {
        if user_idx >= model.user_factors.len() || item_idx >= model.item_factors.len() {
            return 0.0;
        }
        self.dot_product(&model.user_factors[user_idx], &model.item_factors[item_idx])
    }

    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    fn save_model(
        &self,
        model: &MatrixFactorizationModel,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(MODEL_PATH).parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(model)?;
        fs::write(MODEL_PATH, json)?;
        Ok(())
    }

    fn load_model() -> Result<MatrixFactorizationModel, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(MODEL_PATH)?;
        let model: MatrixFactorizationModel = serde_json::from_str(&json)?;
        Ok(model)
    }

    pub async fn track_category_view(
        db_pool: &PgPool,
        user_id: &Uuid,
        item_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO user_category_views (user_id, category_id, view_count, last_viewed)
            SELECT $1, ic.category_id, 1, CURRENT_TIMESTAMP
            FROM item_categories ic
            WHERE ic.item_id = $2
            ON CONFLICT (user_id, category_id)
            DO UPDATE SET
                view_count = user_category_views.view_count + 1,
                last_viewed = CURRENT_TIMESTAMP
            "#,
            user_id,
            item_id
        )
        .execute(db_pool)
        .await?;
        Ok(())
    }
}

impl Default for RecommendationService {
    fn default() -> Self {
        Self::new()
    }
}
