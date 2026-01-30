use axum::{Json, extract::State, response::IntoResponse};
use http::StatusCode;
use tracing::{error, info, warn};

use std::collections::HashMap;

use crate::{
    api::{
        auth::JwtClaims,
        sign::{
            models::{SignPayload, SignResponse, UserSignResponse},
            services::{check_cookie_exists, get_cookies, sign_fn},
        },
        users::{get_user_by_id, get_users_by_ulids},
    },
    misc::GlobalState,
};

#[utoipa::path(
    post,
    path = "/api/sign",
    description = "Sign cookies for the provided ULIDs and URL",
    request_body = SignPayload,
    responses(
        (status = 200, description = "Cookies signed successfully", body = Vec<UserSignResponse>),
        (status = 400, description = "No users found for the provided ULIDs"),
        (status = 401, description = "Unauthorized - Invalid or missing JWT token"),
        (status = 404, description = "No cookies found for today"),
    ),
    tag = "Sign"
)]
pub async fn sign(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    Json(payload): Json<SignPayload>,
) -> impl IntoResponse {
    let cookies = match get_cookies(&state) {
        Ok(Some(cookies)) => cookies,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "No cookies found for today").into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error fetching cookies: {}", err),
            )
                .into_response();
        }
    };

    let users = match get_users_by_ulids(&state, &payload.ulids) {
        Ok(users) => users,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error fetching users: {}", err),
            )
                .into_response();
        }
    };

    if users.len() != payload.ulids.len() {
        error!(
            "Mismatch in number of users found: expected {}, found {}",
            payload.ulids.len(),
            users.len()
        );
        return (
            StatusCode::BAD_REQUEST,
            "No users found for the provided ULIDs",
        )
            .into_response();
    }

    let username_by_id: HashMap<String, String> = users.iter().map(|u| (u.id.clone(), u.username.clone())).collect();
    let signing_result = sign_fn(cookies, users, &payload.url).await;
    println!("Signing result: {:?}", signing_result);

    match signing_result {
        Ok(res) => {
            let validated: Vec<String> = res
                .iter()
                .filter(|r| r.response == SignResponse::Success)
                .filter_map(|r| username_by_id.get(&r.ulid).cloned())
                .collect();
            let failed: Vec<(String, String)> = res
                .iter()
                .filter(|r| r.response != SignResponse::Success)
                .filter_map(|r| username_by_id.get(&r.ulid).map(|u| (u.clone(), sign_response_to_message(&r.response))))
                .collect();
            let initiator_username = get_user_by_id(&state, &jwt_user.sub)
                .ok()
                .flatten()
                .map(|u| u.username)
                .unwrap_or_else(|| jwt_user.sub.to_string());

            if let Some(ref webhook_url) = state.sign_webhook_url {
                if !validated.is_empty() {
                    let url = webhook_url.clone();
                    let validated_clone = validated.clone();
                    let failed_clone = failed.clone();
                    let payload_url = payload.url.clone();
                    let initiator = initiator_username.clone();
                    tokio::spawn(async move {
                        send_sign_webhook_bilan(&url, &validated_clone, &failed_clone, &payload_url, &initiator).await;
                    });
                }
            }
            (StatusCode::OK, Json(res)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/sign/status",
    description = "Check the status of cookies for today",
    responses(
        (status = 200, description = "Cookies exist for today"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No cookies found for today"),
    ),
    tag = "Sign"
)]
pub async fn status(State(state): State<GlobalState>) -> impl IntoResponse {
    info!("Checking status...");
    let today = chrono::Local::now().date_naive();
    match check_cookie_exists(&state, today) {
        Ok(true) => (StatusCode::OK, "Cookies exist for today".to_string()).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            "No cookies found for today".to_string(),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error checking cookies: {}", err),
        )
            .into_response(),
    }
}

fn sign_response_to_message(r: &SignResponse) -> String {
    match r {
        SignResponse::Success => "Succès".to_string(),
        SignResponse::TokenExpired => "Token expiré".to_string(),
        SignResponse::TokenNotFound => "Token non trouvé".to_string(),
        SignResponse::AlreadySigned => "Déjà signé".to_string(),
        SignResponse::UnknownError => "Erreur inconnue".to_string(),
        SignResponse::ServiceUnavailable => "Service indisponible".to_string(),
    }
}

/// Envoie un webhook bilan après signature multiple.
/// Discord : payload { "content": "message lisible" } avec lancé par, URL, validés, échecs.
/// Autres URLs : payload JSON structuré { "event", "initiated_by", "url", "validated", "failed" }.
async fn send_sign_webhook_bilan(
    webhook_url: &str,
    validated: &[String],
    failed: &[(String, String)],
    url: &str,
    initiated_by: &str,
) {
    let is_discord = webhook_url.to_lowercase().contains("discord.com");
    let body: serde_json::Value = if is_discord {
        let mut parts: Vec<String> = Vec::new();
        parts.push("**Bilan signature**".to_string());
        parts.push(format!("**Lancé par :** {}", initiated_by));
        parts.push(format!("**URL :** {}", url));
        if !validated.is_empty() {
            parts.push(format!("✅ **Validés :** {}.", validated.join(", ")));
        }
        if !failed.is_empty() {
            let failed_list: Vec<String> = failed
                .iter()
                .map(|(u, m)| format!("{} ({})", u, m))
                .collect();
            parts.push(format!("❌ **Échecs :** {}.", failed_list.join(" ; ")));
        }
        let content = parts.join("\n");
        let content = if content.len() > 2000 {
            format!("{}…", &content[..1997])
        } else {
            content
        };
        serde_json::json!({ "content": content })
    } else {
        serde_json::json!({
            "event": "sign_multi",
            "initiated_by": initiated_by,
            "url": url,
            "validated": validated,
            "failed": failed.iter().map(|(u, m)| serde_json::json!({ "username": u, "message": m })).collect::<Vec<_>>(),
        })
    };
    let client = match reqwest::Client::builder().build() {
        Ok(c) => c,
        Err(e) => {
            error!("Webhook Sign: impossible de créer le client HTTP: {}", e);
            return;
        }
    };
    match client.post(webhook_url).json(&body).send().await {
        Ok(res) if !res.status().is_success() => {
            warn!("Webhook Sign: statut {} pour {}", res.status(), webhook_url);
        }
        Err(e) => {
            warn!("Webhook Sign: erreur envoi vers {}: {}", webhook_url, e);
        }
        _ => {
            info!("Webhook Sign: bilan envoyé (validés: {}, échecs: {})", validated.len(), failed.len());
        }
    }
}
