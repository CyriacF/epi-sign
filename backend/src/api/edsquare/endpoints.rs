use axum::{Json, extract::{State, Query}, response::IntoResponse};
use http::StatusCode;
use tracing::{error, debug, info, warn};
use ulid::Ulid;
use chrono::NaiveDate;

use crate::{
    api::{
        auth::JwtClaims,
        users::{get_user_by_id, get_all_users},
        edsquare::models::{
            ValidateEdsquarePayload,
            ValidateEdsquareResponse,
            ValidateEdsquareMultiPayload,
            ValidateEdsquareMultiResponse,
            EdsquareUserValidationResult,
            SaveEdsquareCookiesPayload,
            LoginEdsquarePayload,
            LoginEdsquareResponse,
            EdsquareStatusResponse,
            EdsquareEligibleUser,
            EdsquareEligibleUsersResponse,
            EdsquarePlanningEventsResponse,
            PlanningEventsForUsersPayload,
            PlanningEventsForUsersResponse,
            UserPlanningEvents,
        },
        edsquare::services::{
            validate_edsquare_code,
            save_edsquare_cookies,
            login_edsquare,
            login_edsquare_with_saved,
            get_edsquare_cookies,
            get_edsquare_credentials,
            fetch_planning_events,
        },
    },
    misc::GlobalState,
};

#[derive(serde::Deserialize)]
pub struct PlanningEventsQuery {
    pub date: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/edsquare/validate",
    description = "Validate an EDSquare code using the user's signature",
    request_body = ValidateEdsquarePayload,
    responses(
        (status = 200, description = "Code validated successfully", body = ValidateEdsquareResponse),
        (status = 400, description = "Invalid code or missing signature"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found or signature not set"),
    ),
    tag = "EDSquare"
)]
pub async fn validate_edsquare(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    Json(payload): Json<ValidateEdsquarePayload>,
) -> impl IntoResponse {
    let user = match get_user_by_id(&state, &jwt_user.sub) {
        Ok(Some(user)) => {
            debug!("User found: id={}, username={}, has_signature={}", 
                user.id, 
                user.username, 
                user.signature_manuscrite.is_some()
            );
            if let Some(ref sig) = user.signature_manuscrite {
                debug!("Signature length: {} characters", sig.len());
            }
            user
        },
        Ok(None) => {
            error!("User not found: id={}", jwt_user.sub);
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        },
        Err(e) => {
            error!("Error fetching user: {:?}", e);
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        },
    };

    let signature = match &user.signature_manuscrite {
        Some(sig) => {
            info!("Signature found for user {}: {} characters", user.username, sig.len());
            sig
        },
        None => {
            error!("Signature not found for user {} (id: {})", user.username, user.id);
            return (StatusCode::NOT_FOUND, "Signature not set. Please create a signature first.").into_response();
        },
    };

    if payload.planning_event_id.is_empty() {
        return (StatusCode::BAD_REQUEST, "planning_event_id is required").into_response();
    }

    match validate_edsquare_code(&payload.code, &payload.planning_event_id, signature, &jwt_user.sub.to_string(), &state).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Error validating EDSquare code: {}", e);
            if e.contains("cookie") || e.contains("session") {
                (StatusCode::NOT_FOUND, e).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Error validating code: {}", e)).into_response()
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/edsquare/validate-multi",
    description = "Validate an EDSquare code for multiple users using their signatures",
    request_body = ValidateEdsquareMultiPayload,
    responses(
        (status = 200, description = "Codes validated successfully", body = ValidateEdsquareMultiResponse),
        (status = 400, description = "Invalid code, planning_event_id or user list"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "EDSquare"
)]
pub async fn validate_edsquare_multi(
    State(state): State<GlobalState>,
    _jwt_user: JwtClaims,
    Json(payload): Json<ValidateEdsquareMultiPayload>,
) -> impl IntoResponse {
    let use_per_user_codes = payload.user_codes.as_ref().map(|m| !m.is_empty()).unwrap_or(false);
    info!(
        "validate-multi: user_ids={}, use_per_user_codes={}, user_codes_keys={}",
        payload.user_ids.len(),
        use_per_user_codes,
        payload.user_codes.as_ref().map(|m| m.len()).unwrap_or(0)
    );
    if !use_per_user_codes && payload.code.is_empty() {
        return (StatusCode::BAD_REQUEST, "code is required").into_response();
    }
    if payload.planning_event_id.is_empty() && payload.user_planning_event_ids.as_ref().map(|m| m.is_empty()).unwrap_or(true) {
        return (StatusCode::BAD_REQUEST, "planning_event_id or user_planning_event_ids is required").into_response();
    }
    if payload.user_ids.is_empty() {
        return (StatusCode::BAD_REQUEST, "user_ids must not be empty").into_response();
    }

    let mut results: Vec<EdsquareUserValidationResult> = Vec::new();

    for user_id in payload.user_ids.iter() {
        // Convertir l'ID (string) en Ulid pour réutiliser get_user_by_id
        let ulid = match Ulid::from_string(user_id) {
            Ok(id) => id,
            Err(e) => {
                warn!("Invalid user_id '{}' in multi-validate payload: {}", user_id, e);
                results.push(EdsquareUserValidationResult {
                    user_id: user_id.clone(),
                    username: "<invalid id>".to_string(),
                    success: false,
                    message: "Invalid user id".to_string(),
                });
                continue;
            }
        };

        // Récupérer l'utilisateur
        let user = match get_user_by_id(&state, &ulid) {
            Ok(Some(user)) => user,
            Ok(None) => {
                warn!("User not found for multi-validate: {}", user_id);
                results.push(EdsquareUserValidationResult {
                    user_id: user_id.clone(),
                    username: "<unknown>".to_string(),
                    success: false,
                    message: "User not found".to_string(),
                });
                continue;
            }
            Err(e) => {
                error!("Error fetching user {} for multi-validate: {:?}", user_id, e);
                results.push(EdsquareUserValidationResult {
                    user_id: user_id.clone(),
                    username: "<error>".to_string(),
                    success: false,
                    message: "Error fetching user".to_string(),
                });
                continue;
            }
        };

        // Vérifier la signature
        let signature = match &user.signature_manuscrite {
            Some(sig) => sig,
            None => {
                warn!("Signature not set for user {} ({}) in multi-validate", user.username, user.id);
                results.push(EdsquareUserValidationResult {
                    user_id: user.id.clone(),
                    username: user.username.clone(),
                    success: false,
                    message: "Signature not set. Please create a signature first.".to_string(),
                });
                continue;
            }
        };

        // Event ID pour cet utilisateur : override par user_planning_event_ids si présent
        let planning_event_id = payload
            .user_planning_event_ids
            .as_ref()
            .and_then(|m| m.get(&user.id))
            .map(|s| s.as_str())
            .unwrap_or(payload.planning_event_id.as_str());

        // Code pour cet utilisateur : override par user_codes si présent (cours différents = code différent)
        let code = payload
            .user_codes
            .as_ref()
            .and_then(|m| m.get(&user.id))
            .map(|s| s.as_str())
            .unwrap_or(payload.code.as_str());

        if code.len() != 6 {
            results.push(EdsquareUserValidationResult {
                user_id: user.id.clone(),
                username: user.username.clone(),
                success: false,
                message: format!("Le code doit contenir 6 chiffres, reçu: {} caractères", code.len()),
            });
            continue;
        }

        // Appeler la logique de validation existante pour cet utilisateur
        match validate_edsquare_code(
            code,
            planning_event_id,
            signature,
            &user.id,
            &state,
        ).await {
            Ok(_resp) => {
                info!("EDSquare code validated successfully for user {} ({})", user.username, user.id);
                results.push(EdsquareUserValidationResult {
                    user_id: user.id.clone(),
                    username: user.username.clone(),
                    success: true,
                    message: "Code validé avec succès".to_string(),
                });
            }
            Err(e) => {
                error!("Error validating EDSquare code for user {} ({}): {}", user.username, user.id, e);
                results.push(EdsquareUserValidationResult {
                    user_id: user.id.clone(),
                    username: user.username.clone(),
                    success: false,
                    message: e,
                });
            }
        }
    }

    if results.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Aucune validation n'a pu être effectuée (utilisateurs invalides ou sans signature)".to_string(),
        )
            .into_response();
    }

    let global_success = results.iter().all(|r| r.success);
    let response = ValidateEdsquareMultiResponse {
        global_success,
        results,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[utoipa::path(
    post,
    path = "/api/edsquare/cookies",
    description = "Save EDSquare cookies for today",
    request_body = SaveEdsquareCookiesPayload,
    responses(
        (status = 200, description = "Cookies saved successfully"),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "EDSquare"
)]
pub async fn save_edsquare_cookies_endpoint(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    Json(payload): Json<SaveEdsquareCookiesPayload>,
) -> impl IntoResponse {
    match save_edsquare_cookies(&state, &jwt_user.sub.to_string(), &payload.cookies) {
        Ok(_) => (StatusCode::OK, "Cookies EDSquare sauvegardés avec succès").into_response(),
        Err(e) => {
            error!("Error saving EDSquare cookies: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur lors de la sauvegarde: {}", e)).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/edsquare/login",
    description = "Login to EDSquare and save cookies automatically",
    request_body = LoginEdsquarePayload,
    responses(
        (status = 200, description = "Login successful and cookies saved", body = LoginEdsquareResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "EDSquare"
)]
pub async fn login_edsquare_endpoint(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    Json(payload): Json<LoginEdsquarePayload>,
) -> impl IntoResponse {
    if payload.email.is_empty() || payload.password.is_empty() {
        return (StatusCode::BAD_REQUEST, "Email et mot de passe requis").into_response();
    }

    match login_edsquare(&payload.email, &payload.password, &jwt_user.sub.to_string(), &state).await {
        Ok(response) => {
            info!("Login EDSquare réussi pour: {}", payload.email);
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(e) => {
            error!("Error logging in to EDSquare: {}", e);
            // Toutes les erreurs de connexion doivent retourner un status d'erreur
            // pour que le frontend puisse les détecter
            if e.contains("identifiants invalides") 
                || e.contains("Échec de la connexion") 
                || e.contains("invalides") {
                (StatusCode::BAD_REQUEST, e).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur lors de la connexion: {}", e)).into_response()
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/edsquare/login-saved",
    description = "Login to EDSquare using previously saved credentials for the current user",
    responses(
        (status = 200, description = "Login successful and cookies saved", body = LoginEdsquareResponse),
        (status = 400, description = "No saved credentials or invalid credentials"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "EDSquare"
)]
pub async fn login_edsquare_with_saved_endpoint(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
) -> impl IntoResponse {
    let user_id_str = jwt_user.sub.to_string();

    match login_edsquare_with_saved(&user_id_str, &state).await {
        Ok(response) => {
            info!(
                "Login EDSquare avec identifiants sauvegardés réussi pour l'utilisateur {}",
                user_id_str
            );
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(
                "Error logging in to EDSquare with saved credentials for user {}: {}",
                user_id_str, e
            );
            if e.contains("Aucun identifiant EDSquare enregistré") {
                (StatusCode::BAD_REQUEST, e).into_response()
            } else if e.contains("identifiants invalides") || e.contains("Échec de la connexion") {
                (StatusCode::BAD_REQUEST, e).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "Erreur lors de la connexion avec identifiants sauvegardés: {}",
                        e
                    ),
                )
                    .into_response()
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/edsquare/status",
    description = "Get EDSquare status (signature and cookies)",
    responses(
        (status = 200, description = "Status retrieved successfully", body = EdsquareStatusResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "EDSquare"
)]
pub async fn get_edsquare_status(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
) -> impl IntoResponse {
    let user_id_str = jwt_user.sub.to_string();
    debug!("Vérification du statut EDSquare pour l'utilisateur: {}", user_id_str);
    
    let user = match get_user_by_id(&state, &jwt_user.sub) {
        Ok(Some(user)) => {
            debug!("Utilisateur trouvé: id={}, username={}, has_signature={}", 
                user.id, user.username, user.signature_manuscrite.is_some());
            user
        },
        Ok(None) => {
            error!("Utilisateur non trouvé: id={}", jwt_user.sub);
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        },
        Err(e) => {
            error!("Erreur lors de la récupération de l'utilisateur: {:?}", e);
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        },
    };

    let has_signature = user.signature_manuscrite.is_some();
    info!("Statut signature pour {}: {}", user.username, has_signature);

    let has_cookies = match get_edsquare_cookies(&state, &user_id_str) {
        Ok(Some(cookies)) => {
            info!("Cookies EDSquare trouvés pour {}: {} cookies valides", user.username, cookies.len());
            true
        },
        Ok(None) => {
            warn!("Aucun cookie EDSquare valide trouvé pour {}", user.username);
            false
        },
        Err(e) => {
            error!("Erreur lors de la récupération des cookies EDSquare pour {}: {}", user.username, e);
            false
        },
    };

    let has_saved_credentials = get_edsquare_credentials(&state, &user_id_str)
        .ok()
        .and_then(|o| o)
        .is_some();
    if has_saved_credentials && !has_cookies {
        info!("Identifiants EDSquare sauvegardés pour {} : reconnexion auto possible", user.username);
    }

    let response = EdsquareStatusResponse {
        has_signature,
        has_cookies,
        has_saved_credentials,
        is_ready: has_signature && (has_cookies || has_saved_credentials),
    };

    info!("Statut EDSquare final pour {}: has_signature={}, has_cookies={}, is_ready={}", 
        user.username, has_signature, has_cookies, response.is_ready);

    (StatusCode::OK, Json(response)).into_response()
}

#[utoipa::path(
    get,
    path = "/api/edsquare/eligible-users",
    description = "Get all users that are ready for EDSquare (signature + valid cookies)",
    responses(
        (status = 200, description = "Eligible users retrieved successfully", body = EdsquareEligibleUsersResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "EDSquare"
)]
pub async fn get_edsquare_eligible_users(
    State(state): State<GlobalState>,
    _jwt_user: JwtClaims,
) -> impl IntoResponse {
    // Récupérer tous les utilisateurs
    let users = match get_all_users(&state) {
        Ok(users) => users,
        Err(e) => {
            error!("Error fetching users for EDSquare eligibility: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching users").into_response();
        }
    };

    let mut eligible: Vec<EdsquareEligibleUser> = Vec::new();

    for user in users {
        let has_signature = user.signature_manuscrite.is_some();
        if !has_signature {
            continue;
        }

        let user_id_str = user.id.clone();
        let has_cookies = match get_edsquare_cookies(&state, &user_id_str) {
            Ok(Some(cookies)) => {
                info!(
                    "EDSquare cookies found for user {} ({}): {} cookies valides",
                    user.username,
                    user.id,
                    cookies.len()
                );
                true
            }
            Ok(None) => false,
            Err(e) => {
                error!(
                    "Error fetching EDSquare cookies for user {} ({}): {}",
                    user.username,
                    user.id,
                    e
                );
                false
            }
        };

        let has_saved_credentials = get_edsquare_credentials(&state, &user_id_str)
            .ok()
            .and_then(|o| o)
            .is_some();

        if has_cookies || has_saved_credentials {
            eligible.push(EdsquareEligibleUser {
                id: user.id.clone(),
                username: user.username.clone(),
            });
        }
    }

    // Optionnel : ordonner par username pour plus de lisibilité
    eligible.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));

    let response = EdsquareEligibleUsersResponse { users: eligible };
    (StatusCode::OK, Json(response)).into_response()
}

#[utoipa::path(
    get,
    path = "/api/edsquare/planning-events",
    description = "Get planning events for a date from EDSquare (json_dashboard). Uses current user's EDSquare cookies.",
    params(
        ("date" = Option<String>, Query, description = "Date YYYY-MM-DD (default: today)")
    ),
    responses(
        (status = 200, description = "Events retrieved successfully", body = EdsquarePlanningEventsResponse),
        (status = 400, description = "Invalid date format"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No EDSquare cookies"),
    ),
    tag = "EDSquare"
)]
pub async fn get_planning_events(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    Query(query): Query<PlanningEventsQuery>,
) -> impl IntoResponse {
    let date = match &query.date {
        Some(d) => match NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Format de date invalide. Utilisez YYYY-MM-DD.",
                )
                    .into_response()
            }
        },
        None => chrono::Utc::now().date_naive(),
    };

    match fetch_planning_events(&state, &jwt_user.sub.to_string(), date).await {
        Ok(events) => {
            let response = EdsquarePlanningEventsResponse { events };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            if e.contains("cookie") || e.contains("Session EDSquare expirée") {
                (StatusCode::NOT_FOUND, e).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/edsquare/planning-events-for-users",
    description = "Get planning events for a date for each of the given users (uses each user's EDSquare cookies).",
    request_body = PlanningEventsForUsersPayload,
    responses(
        (status = 200, description = "Events per user", body = PlanningEventsForUsersResponse),
        (status = 400, description = "Invalid date or empty user_ids"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "EDSquare"
)]
pub async fn get_planning_events_for_users(
    State(state): State<GlobalState>,
    _jwt_user: JwtClaims,
    Json(payload): Json<PlanningEventsForUsersPayload>,
) -> impl IntoResponse {
    if payload.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "user_ids ne doit pas être vide".to_string(),
        )
            .into_response();
    }

    let date = match NaiveDate::parse_from_str(payload.date.as_str(), "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "Format de date invalide. Utilisez YYYY-MM-DD.".to_string(),
            )
                .into_response();
        }
    };

    let mut user_events: Vec<UserPlanningEvents> = Vec::new();

    for user_id in &payload.user_ids {
        let ulid = match Ulid::from_string(user_id) {
            Ok(id) => id,
            Err(_) => {
                user_events.push(UserPlanningEvents {
                    user_id: user_id.clone(),
                    username: "<invalid id>".to_string(),
                    events: vec![],
                    error: Some("ID utilisateur invalide".to_string()),
                });
                continue;
            }
        };

        let username = match get_user_by_id(&state, &ulid) {
            Ok(Some(u)) => u.username.clone(),
            Ok(None) => "<unknown>".to_string(),
            Err(_) => "<error>".to_string(),
        };

        match fetch_planning_events(&state, user_id, date).await {
            Ok(events) => {
                user_events.push(UserPlanningEvents {
                    user_id: user_id.clone(),
                    username,
                    events,
                    error: None,
                });
            }
            Err(e) => {
                user_events.push(UserPlanningEvents {
                    user_id: user_id.clone(),
                    username,
                    events: vec![],
                    error: Some(e),
                });
            }
        }
    }

    let response = PlanningEventsForUsersResponse { user_events };
    (StatusCode::OK, Json(response)).into_response()
}
