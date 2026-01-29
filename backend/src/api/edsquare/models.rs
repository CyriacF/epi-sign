use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::api::sign::CookieItem;

#[derive(Deserialize, ToSchema, Debug)]
pub struct ValidateEdsquarePayload {
    #[schema(example = "000000")]
    pub code: String,
    #[schema(example = "199289")]
    pub planning_event_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct ValidateEdsquareResponse {
    pub success: bool,
    pub message: String,
    pub code: String,
    pub planning_event_id: Option<String>,
}

/// Payload pour la validation multi-utilisateurs côté EDSquare
#[derive(Deserialize, ToSchema, Debug)]
pub struct ValidateEdsquareMultiPayload {
    #[schema(example = "000000")]
    pub code: String,
    #[schema(example = "199289")]
    pub planning_event_id: String,
    /// Liste des IDs d'utilisateurs à valider
    pub user_ids: Vec<String>,
}

/// Résultat de validation pour un utilisateur donné
#[derive(Serialize, ToSchema)]
pub struct EdsquareUserValidationResult {
    pub user_id: String,
    pub username: String,
    pub success: bool,
    pub message: String,
}

/// Réponse globale pour la validation multi-utilisateurs
#[derive(Serialize, ToSchema)]
pub struct ValidateEdsquareMultiResponse {
    /// true si toutes les validations ont réussi
    pub global_success: bool,
    pub results: Vec<EdsquareUserValidationResult>,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct SaveEdsquareCookiesPayload {
    pub cookies: Vec<CookieItem>,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct LoginEdsquarePayload {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginEdsquareResponse {
    pub success: bool,
    pub message: String,
}

// Réutiliser CookieItem de sign pour les cookies EDSquare
pub type EdsquareCookieItem = CookieItem;

#[derive(Serialize, ToSchema)]
pub struct EdsquareStatusResponse {
    pub has_signature: bool,
    pub has_cookies: bool,
    pub is_ready: bool,
}

/// Utilisateur éligible pour la signature EDSquare (signature + cookies valides)
#[derive(Serialize, ToSchema)]
pub struct EdsquareEligibleUser {
    pub id: String,
    pub username: String,
}

#[derive(Serialize, ToSchema)]
pub struct EdsquareEligibleUsersResponse {
    pub users: Vec<EdsquareEligibleUser>,
}

/// Événement du planning EDSquare (json_dashboard)
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct EdsquarePlanningEvent {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub target: Option<String>,
    pub start: String,
    pub end: String,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub registrable: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct EdsquarePlanningEventsResponse {
    pub events: Vec<EdsquarePlanningEvent>,
}
