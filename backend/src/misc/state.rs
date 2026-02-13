use std::sync::Arc;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
};
use tokio::sync::RwLock;
use chrono::NaiveDate;

use crate::api::edsquare::models::EdsquarePlanningEvent;

/// Cache pour les cours EDSquare par (user_id, date) avec expiration
#[derive(Clone)]
pub struct PlanningEventsCacheEntry {
    pub events: Vec<EdsquarePlanningEvent>,
    pub cached_at: SystemTime,
}

/// Cache en mémoire pour les cours EDSquare (durée de vie: 5 minutes)
type PlanningEventsCache = Arc<RwLock<HashMap<(String, NaiveDate), PlanningEventsCacheEntry>>>;

#[derive(Clone)]
pub struct GlobalState {
    pub db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    pub register_key: String,
    /// Clé optionnelle pour les actions admin (ex: suppression d'utilisateurs). Si définie, requiert le header X-Admin-Key sur DELETE /api/admin/users/:id.
    pub admin_key: Option<String>,
    /// URL optionnelle pour envoyer un webhook bilan après validation EDSquare multi-utilisateurs (ex: Discord, Slack, API custom).
    pub edsquare_webhook_url: Option<String>,
    /// URL optionnelle pour envoyer un webhook bilan après signature multiple (ex: Discord, Slack, API custom).
    pub sign_webhook_url: Option<String>,
    /// Cache pour les cours EDSquare (évite de spammer l'API)
    pub edsquare_planning_cache: PlanningEventsCache,
}

impl GlobalState {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let register_key = std::env::var("REGISTER_KEY").expect("REGISTER_KEY must be set");
        let admin_key = std::env::var("ADMIN_KEY").ok().filter(|s| !s.trim().is_empty());
        let edsquare_webhook_url = std::env::var("EDSQUARE_WEBHOOK_URL").ok().filter(|s| !s.trim().is_empty());
        let sign_webhook_url = std::env::var("SIGN_WEBHOOK_URL").ok().filter(|s| !s.trim().is_empty());
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let db_pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .expect("Failed to create database connection pool");

        GlobalState {
            db_pool: Arc::new(db_pool),
            register_key,
            admin_key,
            edsquare_webhook_url,
            sign_webhook_url,
            edsquare_planning_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_db_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, PoolError> {
        self.db_pool.get()
    }
}
