use crate::api::{
    edsquare::models::{
        ValidateEdsquareResponse, EdsquareCookieItem, LoginEdsquareResponse,
        EdsquarePlanningEvent,
    },
    sign::CookieItem,
};
use crate::misc::GlobalState;
use http::header::COOKIE;
use http::StatusCode;
use tracing::{error, info, warn, debug};
use chrono::NaiveDate;
use serde_json::Value;
use diesel::prelude::*;
use urlencoding::encode;

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::edsquare_cookies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)] // Les champs id, date et user_id sont utilisés dans les requêtes Diesel
struct EdsquareCookie {
    id: String,
    user_id: String,
    date: NaiveDate,
    cookie_data: Value,
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::edsquare_credentials)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct EdsquareCredential {
    id: String,
    #[allow(dead_code)]
    user_id: String,
    email: String,
    password: String,
}

/// Sauvegarde ou met à jour les identifiants EDSquare pour un utilisateur.
pub fn save_edsquare_credentials(
    state: &GlobalState,
    user_id_param: &str,
    email_param: &str,
    password_param: &str,
) -> Result<(), String> {
    use crate::schema::edsquare_credentials::dsl::*;
    use ulid::Ulid;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to get database connection".into()),
    };

    let existing = edsquare_credentials
        .filter(user_id.eq(user_id_param))
        .select(EdsquareCredential::as_select())
        .first::<EdsquareCredential>(&mut conn)
        .optional()
        .map_err(|e| {
            error!("Erreur lors de la récupération des identifiants EDSquare: {}", e);
            format!("Database error when fetching EDSquare credentials: {}", e)
        })?;

    if let Some(cred) = existing {
        info!(
            "Mise à jour des identifiants EDSquare pour l'utilisateur {}",
            user_id_param
        );
        diesel::update(edsquare_credentials.filter(id.eq(cred.id)))
            .set((email.eq(email_param), password.eq(password_param)))
            .execute(&mut conn)
            .map_err(|e| {
                error!("Erreur lors de la mise à jour des identifiants EDSquare: {}", e);
                format!("Database error when updating EDSquare credentials: {}", e)
            })?;
    } else {
        let cred_id = Ulid::new().to_string();
        info!(
            "Sauvegarde de nouveaux identifiants EDSquare pour l'utilisateur {}",
            user_id_param
        );
        diesel::insert_into(edsquare_credentials)
            .values((
                id.eq(cred_id),
                user_id.eq(user_id_param),
                email.eq(email_param),
                password.eq(password_param),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                error!("Erreur lors de l'insertion des identifiants EDSquare: {}", e);
                format!("Database error when inserting EDSquare credentials: {}", e)
            })?;
    }

    Ok(())
}

/// Récupère les identifiants EDSquare sauvegardés pour un utilisateur.
pub fn get_edsquare_credentials(
    state: &GlobalState,
    user_id_param: &str,
) -> Result<Option<(String, String)>, String> {
    use crate::schema::edsquare_credentials::dsl::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to get database connection".into()),
    };

    let existing = edsquare_credentials
        .filter(user_id.eq(user_id_param))
        .select(EdsquareCredential::as_select())
        .first::<EdsquareCredential>(&mut conn)
        .optional()
        .map_err(|e| {
            error!("Erreur lors de la récupération des identifiants EDSquare: {}", e);
            format!("Database error when fetching EDSquare credentials: {}", e)
        })?;

    Ok(existing.map(|cred| (cred.email, cred.password)))
}

pub fn get_edsquare_cookies(state: &GlobalState, user_id_param: &str) -> Result<Option<Vec<EdsquareCookieItem>>, String> {
    use crate::schema::edsquare_cookies;
    use crate::schema::edsquare_cookies::dsl::*;

    let current_date = chrono::Utc::now().date_naive();

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to get database connection".into()),
    };

    info!("Récupération des cookies EDSquare pour l'utilisateur {} et la date: {}", user_id_param, current_date);

    let cookie_record = edsquare_cookies::table
        .filter(user_id.eq(user_id_param))
        .filter(date.eq(current_date))
        .select(EdsquareCookie::as_select())
        .first::<EdsquareCookie>(&mut conn)
        .optional();

    let cookie = match cookie_record {
        Ok(Some(cookie)) => {
            info!("Cookies EDSquare trouvés pour l'utilisateur {} à la date {}", user_id_param, current_date);
            cookie
        },
        Ok(None) => {
            warn!("Aucun cookie EDSquare trouvé pour l'utilisateur {} à la date {}", user_id_param, current_date);
            return Ok(None);
        }
        Err(e) => {
            error!("Erreur lors de la récupération des cookies: {}", e);
            return Err(format!("Database error when fetching EDSquare cookies: {}", e));
        }
    };

    // Désérialiser directement depuis le Value JSON
    match serde_json::from_value::<Vec<EdsquareCookieItem>>(cookie.cookie_data.clone()) {
        Ok(cookie_items) => {
            let total_cookies = cookie_items.len();
            info!("{} cookies EDSquare parsés avec succès pour l'utilisateur {}", total_cookies, user_id_param);
            
            // Vérifier la durée de vie des cookies
            let now = chrono::Utc::now().timestamp();
            let valid_cookies: Vec<EdsquareCookieItem> = cookie_items
                .into_iter()
                .filter(|c| {
                    // Si le cookie n'a pas de date d'expiration, on le garde
                    // Sinon, on vérifie qu'il n'est pas expiré
                    match c.expires {
                        Some(expires) => {
                            let is_valid = expires > now;
                            if !is_valid {
                                warn!("Cookie '{}' expiré (expires: {}, now: {})", c.name, expires, now);
                            }
                            is_valid
                        }
                        None => {
                            // Pas de date d'expiration explicite, on garde le cookie
                            // Mais on vérifie qu'il n'est pas trop vieux (plus de 7 jours)
                            // Les sessions EDSquare expirent généralement après quelques heures/jours
                            true
                        }
                    }
                })
                .collect();
            
            if valid_cookies.is_empty() {
                warn!("Tous les cookies EDSquare sont expirés pour l'utilisateur {}", user_id_param);
                return Ok(None);
            }
            
            if valid_cookies.len() < total_cookies {
                info!("{} cookies valides sur {} pour l'utilisateur {}", valid_cookies.len(), total_cookies, user_id_param);
            }
            
            info!("Retour de {} cookies valides pour l'utilisateur {}", valid_cookies.len(), user_id_param);
            Ok(Some(valid_cookies))
        },
        Err(e) => {
            error!("Erreur lors du parsing des cookies depuis JSON: {}", e);
            error!("Données des cookies (preview): {:?}", serde_json::to_string(&cookie.cookie_data).unwrap_or_else(|_| "Erreur de sérialisation".to_string()));
            Err(format!("Failed to parse EDSquare cookie JSON: {}", e))
        }
    }
}

/// Invalide les cookies EDSquare stockés pour un utilisateur (date du jour). Force une reconnexion au prochain appel.
pub fn clear_edsquare_cookies_for_user(state: &GlobalState, user_id_param: &str) -> Result<(), String> {
    use crate::schema::edsquare_cookies::dsl::*;
    use diesel::prelude::*;

    let current_date = chrono::Utc::now().date_naive();
    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to get database connection".into()),
    };

    diesel::delete(edsquare_cookies.filter(user_id.eq(user_id_param)).filter(date.eq(current_date)))
        .execute(&mut conn)
        .map_err(|e| {
            error!("Erreur lors de la suppression des cookies EDSquare: {}", e);
            format!("Failed to clear EDSquare cookies: {}", e)
        })?;
    info!("Cookies EDSquare invalidés pour l'utilisateur {} (session expirée)", user_id_param);
    Ok(())
}

/// Récupère les cookies EDSquare ; si aucun ou expirés, tente une reconnexion automatique avec les identifiants sauvegardés.
pub async fn get_edsquare_cookies_or_reconnect(
    state: &GlobalState,
    user_id_param: &str,
) -> Result<Vec<EdsquareCookieItem>, String> {
    match get_edsquare_cookies(state, user_id_param) {
        Ok(Some(cookies)) => return Ok(cookies),
        Ok(None) => {
            info!(
                "Pas de cookie EDSquare valide pour {}, tentative de reconnexion avec identifiants sauvegardés",
                user_id_param
            );
            if let Err(e) = login_edsquare_with_saved(user_id_param, state).await {
                return Err(e);
            }
            match get_edsquare_cookies(state, user_id_param) {
                Ok(Some(cookies)) => Ok(cookies),
                Ok(None) => Err("Reconnexion EDSquare effectuée mais aucun cookie reçu.".into()),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

/// Effectue une tentative de validation EDSquare (cookies déjà récupérés ou à récupérer).
async fn validate_edsquare_code_once(
    code: &str,
    planning_event_id: &str,
    signature: &str,
    user_id: &str,
    state: &GlobalState,
) -> Result<ValidateEdsquareResponse, String> {
    // Récupérer les cookies EDSquare (reconnexion auto si identifiants sauvegardés)
    let cookies = get_edsquare_cookies_or_reconnect(state, user_id).await?;

    let client = match get_reqwest_client() {
        Ok(client) => client,
        Err(e) => {
            return Err(format!("Failed to create HTTP client: {}", e));
        }
    };

    // Construire la chaîne de cookies
    let cookie_str = cookies
        .iter()
        .map(|c| c.to_header_value())
        .collect::<Vec<_>>()
        .join("; ");

    // Récupérer le CSRF token en appelant une page EDSquare qui le fournit dans le HTML
    let csrf_token = match fetch_csrf_token_with_cookies(&cookie_str).await {
        Ok(token_opt) => token_opt,
        Err(e) => {
            if e.contains("Session EDSquare expirée") {
                return Err(e);
            }
            warn!("Impossible de récupérer le CSRF token EDSquare: {}", e);
            None
        }
    };

    // Construire le corps form-urlencoded pour envoyer la signature :
    // - authenticity_token (CSRF Rails) – même valeur que le header X-CSRF-Token
    // - course_user_signature[planning_event_id]
    // - course_user_signature[signature_data] (PNG base64 data:image/png;base64,...)
    // - secret_code_part_1 à secret_code_part_6 (code à 6 chiffres divisé)
    // Note: Les crochets [] doivent être encodés en %5B et %5D, mais les valeurs doivent être encodées séparément
    // L'ordre doit correspondre au curl fourni : authenticity_token, planning_event_id, signature_data, puis les 6 parties du code
    
    // Vérifier que le code a exactement 6 caractères
    if code.len() != 6 {
        return Err(format!("Le code secret doit contenir exactement 6 chiffres, reçu: {} caractères", code.len()));
    }
    
    // Diviser le code en 6 parties
    let code_chars: Vec<char> = code.chars().collect();
    let secret_code_parts = format!(
        "secret_code_part_1={}&secret_code_part_2={}&secret_code_part_3={}&secret_code_part_4={}&secret_code_part_5={}&secret_code_part_6={}",
        encode(&code_chars[0].to_string()),
        encode(&code_chars[1].to_string()),
        encode(&code_chars[2].to_string()),
        encode(&code_chars[3].to_string()),
        encode(&code_chars[4].to_string()),
        encode(&code_chars[5].to_string()),
    );

    // Construire les différentes parties du formulaire dans l'ordre exact du curl
    let mut form_parts: Vec<String> = Vec::new();

    // authenticity_token = même valeur que le header X-CSRF-Token
    if let Some(token) = &csrf_token {
        form_parts.push(format!("authenticity_token={}", encode(token)));
    }

    form_parts.push(format!(
        "course_user_signature%5Bplanning_event_id%5D={}",
        encode(planning_event_id)
    ));

    form_parts.push(format!(
        "course_user_signature%5Bsignature_data%5D={}",
        encode(signature)
    ));

    // Les parties du code (déjà sous forme key=value&key2=value2...)
    form_parts.push(secret_code_parts);

    let form_data = form_parts.join("&");

    info!(
        "Envoi de la requête de signature EDSquare: planning_event_id={}, code={} (divisé en 6 parties), signature_length={}",
        planning_event_id,
        code,
        signature.len()
    );
    debug!(
        "Form data (preview, tronqué): {}{}",
        &form_data.chars().take(200).collect::<String>(),
        if form_data.len() > 200 { "..." } else { "" }
    );
    
    // Endpoint pour envoyer la signature (pas check_secret_code qui sert juste à vérifier le code)
    let endpoint_url = "https://app.edsquare.fr/apps/course_user_signatures";
    info!("Envoi de la requête vers: {}", endpoint_url);
    
    let mut request_builder = client
        .post(endpoint_url)
        .header(COOKIE, &cookie_str)
        .header("Accept", "text/javascript, application/javascript, application/ecmascript, application/x-ecmascript, */*; q=0.01")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7")
        .header("Cache-Control", "no-cache")
        .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
        .header("Origin", "https://app.edsquare.fr")
        .header("Pragma", "no-cache")
        .header("Priority", "u=1, i")
        .header("Referer", "https://app.edsquare.fr/apps/classrooms")
        .header("Sec-Ch-Ua", r#""Not(A:Brand";v="8", "Chromium";v="144", "Google Chrome";v="144""#)
        .header("Sec-Ch-Ua-Mobile", "?0")
        .header("Sec-Ch-Ua-Platform", r#""macOS""#)
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .header("X-Requested-With", "XMLHttpRequest");

    if let Some(token) = &csrf_token {
        request_builder = request_builder.header("X-CSRF-Token", token);
        debug!("CSRF token inclus dans la requête");
    } else {
        warn!("Aucun CSRF token disponible pour la requête");
    }

    let response = match request_builder.body(form_data.clone()).send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!("HTTP request failed: {}", e);
            return Err(format!("Erreur lors de la requête HTTP: {}", e));
        }
    };

    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();

    info!("Réponse EDSquare: status={}, response_length={}", status, response_text.len());
    // Log brut pour le debug dans les logs Docker (peut être verbeux, mais utile pour diagnostiquer)
    if !response_text.is_empty() {
        info!("Réponse EDSquare brute: {}", response_text);
    }

    match status {
        StatusCode::OK => {
            // Même en 200, EDSquare peut renvoyer un JS avec un toastr d'erreur
            // Exemple : toastr.error("Le code saisi n&#39;est plus valide")
            if let Ok(re) = regex::Regex::new(r#"toastr\.error\("([^"]+)""#) {
                if let Some(caps) = re.captures(&response_text) {
                    let raw_msg = caps.get(1).map(|m| m.as_str()).unwrap_or("Erreur EDSquare");
                    error!(
                        "EDSquare a renvoyé une erreur malgré le status 200: {}",
                        raw_msg
                    );
                    return Err(format!("Erreur EDSquare: {}", raw_msg));
                }
            }

            info!("Code EDSquare validé avec succès");
            Ok(ValidateEdsquareResponse {
                success: true,
                message: "Code validé avec succès".to_string(),
                code: code.to_string(),
                planning_event_id: Some(planning_event_id.to_string()),
            })
        }
        StatusCode::UNAUTHORIZED => {
            error!("Session EDSquare expirée (401)");
            Err("Session EDSquare expirée. Veuillez vous reconnecter.".into())
        }
        StatusCode::NOT_FOUND => {
            error!("Code invalide ou événement non trouvé (404). Réponse: {}", response_text);
            Err(format!("Code invalide ou événement non trouvé. Vérifiez le code et le planning_event_id. Réponse: {}", 
                if response_text.len() > 200 { format!("{}...", &response_text[..200]) } else { response_text }))
        }
        StatusCode::BAD_REQUEST => {
            error!("Requête invalide (400). Réponse: {}", response_text);
            Err(format!("Requête invalide. Vérifiez le format des données. Réponse: {}", 
                if response_text.len() > 200 { format!("{}...", &response_text[..200]) } else { response_text }))
        }
        _ => {
            error!("Unexpected status code: {} - Response: {}", status, 
                if response_text.len() > 200 { format!("{}...", &response_text[..200]) } else { response_text.clone() });
            Err(format!("Erreur lors de la validation: {} - {}", status, 
                if response_text.len() > 200 { format!("{}...", &response_text[..200]) } else { response_text }))
        }
    }
}

pub async fn validate_edsquare_code(
    code: &str,
    planning_event_id: &str,
    signature: &str,
    user_id: &str,
    state: &GlobalState,
) -> Result<ValidateEdsquareResponse, String> {
    let result = validate_edsquare_code_once(code, planning_event_id, signature, user_id, state).await;
    if result.is_ok() {
        return result;
    }
    let err = result.unwrap_err();
    let session_expired = err.contains("Session EDSquare expirée")
        || err.contains("Veuillez vous reconnecter")
        || err.to_lowercase().contains("cookie")
        || err.to_lowercase().contains("session");
    if !session_expired {
        return Err(err);
    }
    info!(
        "Session EDSquare expirée pour {}, invalidation des cookies puis reconnexion et nouvel essai",
        user_id
    );
    if let Err(e) = clear_edsquare_cookies_for_user(state, user_id) {
        warn!("Impossible d'invalider les cookies EDSquare: {}", e);
        return Err(err);
    }
    if let Err(reconnect_err) = login_edsquare_with_saved(user_id, state).await {
        warn!("Reconnexion EDSquare échouée après session expirée: {}", reconnect_err);
        return Err(err);
    }
    validate_edsquare_code_once(code, planning_event_id, signature, user_id, state).await
}

/// Récupère les événements du planning EDSquare pour une date donnée (json_dashboard).
/// Utilise les cookies EDSquare de l'utilisateur (reconnexion auto si identifiants sauvegardés).
pub async fn fetch_planning_events(
    state: &GlobalState,
    user_id_param: &str,
    date: NaiveDate,
) -> Result<Vec<EdsquarePlanningEvent>, String> {
    let cookies = get_edsquare_cookies_or_reconnect(state, user_id_param).await?;

    let cookie_str = cookies
        .iter()
        .map(|c| c.to_header_value())
        .collect::<Vec<_>>()
        .join("; ");

    let start_str = format!("{}T00:00:00+01:00", date);
    let end_date = date.succ_opt().unwrap_or(date);
    let end_str = format!("{}T00:00:00+01:00", end_date);

    let url = format!(
        "https://app.edsquare.fr/apps/planning/json_dashboard?start={}&end={}",
        encode(&start_str),
        encode(&end_str)
    );

    let client = match get_reqwest_client() {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to create HTTP client: {}", e)),
    };

    let response = client
        .get(&url)
        .header(COOKIE, &cookie_str)
        .header("Accept", "*/*")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7")
        .header("Cache-Control", "no-cache")
        .header("Referer", "https://app.edsquare.fr/home")
        .send()
        .await
        .map_err(|e| format!("Erreur lors de la requête planning EDSquare: {}", e))?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        if status == StatusCode::UNAUTHORIZED || body.contains("sign_in") {
            return Err("Session EDSquare expirée. Veuillez vous reconnecter à EDSquare.".into());
        }
        return Err(format!(
            "EDSquare planning a répondu avec le statut {}",
            status
        ));
    }

    let events: Vec<EdsquarePlanningEvent> = serde_json::from_str(&body).map_err(|e| {
        error!("Erreur parsing JSON planning EDSquare: {}", e);
        format!("Réponse EDSquare invalide: {}", e)
    })?;

    info!(
        "Récupération de {} événement(s) EDSquare pour la date {} (user: {})",
        events.len(),
        date,
        user_id_param
    );
    Ok(events)
}

async fn fetch_csrf_token_with_cookies(
    cookie_str: &str,
) -> Result<Option<String>, String> {
    // Créer un client temporaire qui ne suit pas les redirections pour détecter les sessions expirées
    let no_redirect_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("Failed to create HTTP client without redirects: {}", e))?;

    let response = no_redirect_client
        .get("https://app.edsquare.fr/apps/classrooms")
        .header(COOKIE, cookie_str)
        .send()
        .await
        .map_err(|e| format!("Erreur lors de la récupération de la page classrooms: {}", e))?;

    let status = response.status();
    let final_url = response.url().to_string();

    // Vérifier si on est redirigé vers la page de login (session expirée)
    if status.is_redirection() || final_url.contains("/users/sign_in") {
        warn!("Session EDSquare expirée : redirection vers /users/sign_in détectée");
        return Err("Session EDSquare expirée. Veuillez vous reconnecter à EDSquare.".into());
    }

    let html = response.text().await.unwrap_or_default();

    if !status.is_success() {
        warn!(
            "Échec de la récupération de la page classrooms pour CSRF (status: {}, url: {})",
            status, final_url
        );
        return Ok(None);
    }

    if let Some(token) = extract_csrf_token_from_html(&html) {
        info!("CSRF token EDSquare récupéré depuis /apps/classrooms");
        Ok(Some(token))
    } else {
        warn!("Impossible de trouver le CSRF token dans la page /apps/classrooms");
        Ok(None)
    }
}

pub fn save_edsquare_cookies(
    state: &GlobalState,
    user_id_param: &str,
    cookie_items: &[EdsquareCookieItem],
) -> Result<(), String> {
    use crate::schema::edsquare_cookies;
    use crate::schema::edsquare_cookies::dsl::*;
    use diesel::prelude::*;
    use ulid::Ulid;
    use serde_json::json;

    let current_date = chrono::Utc::now().date_naive();
    let cookie_id = Ulid::new().to_string();

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to get database connection".into()),
    };

    // Convertir les cookies en JSON
    let cookie_json = json!(cookie_items);

    // Vérifier si des cookies existent déjà pour cet utilisateur aujourd'hui
    let existing = edsquare_cookies::table
        .filter(user_id.eq(user_id_param))
        .filter(date.eq(current_date))
        .select(EdsquareCookie::as_select())
        .first::<EdsquareCookie>(&mut conn)
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    match existing {
        Some(_) => {
            // Mettre à jour les cookies existants
            info!("Mise à jour des cookies EDSquare existants pour l'utilisateur {} et la date: {}", user_id_param, current_date);
            diesel::update(edsquare_cookies::table
                .filter(user_id.eq(user_id_param))
                .filter(date.eq(current_date)))
                .set(cookie_data.eq(cookie_json))
                .execute(&mut conn)
                .map_err(|e| {
                    error!("Erreur lors de la mise à jour des cookies: {}", e);
                    format!("Failed to update cookies: {}", e)
                })?;
            info!("Cookies EDSquare mis à jour avec succès");
        }
        None => {
            // Insérer de nouveaux cookies
            info!("Insertion de nouveaux cookies EDSquare pour l'utilisateur {} et la date: {}", user_id_param, current_date);
            diesel::insert_into(edsquare_cookies::table)
                .values((
                    id.eq(&cookie_id),
                    user_id.eq(user_id_param),
                    date.eq(current_date),
                    cookie_data.eq(cookie_json),
                ))
                .execute(&mut conn)
                .map_err(|e| {
                    error!("Erreur lors de l'insertion des cookies: {}", e);
                    format!("Failed to insert cookies: {}", e)
                })?;
            info!("Cookies EDSquare insérés avec succès");
        }
    }

    Ok(())
}

fn get_reqwest_client() -> Result<reqwest::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("*/*"),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        reqwest::header::HeaderValue::from_static("fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7"),
    );

    reqwest::Client::builder()
        .cookie_store(true)
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

pub async fn login_edsquare(
    email: &str,
    password: &str,
    user_id_param: &str,
    state: &GlobalState,
) -> Result<LoginEdsquareResponse, String> {
    info!("Tentative de connexion EDSquare pour: {}", email);
    
    let client = match get_reqwest_client() {
        Ok(client) => client,
        Err(e) => return Err(format!("Failed to create HTTP client: {}", e)),
    };

    // Étape 1: Récupérer la page de login pour obtenir le CSRF token
    let login_page = match client
        .get("https://app.edsquare.fr/users/sign_in")
        .send()
        .await
    {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(format!("Failed to fetch login page: {}", response.status()));
            }
            response.text().await.unwrap_or_default()
        }
        Err(e) => {
            return Err(format!("Erreur lors de la récupération de la page de login: {}", e));
        }
    };

    // Extraire le CSRF token depuis le HTML
    let csrf_token = extract_csrf_token_from_html(&login_page)
        .ok_or_else(|| "Impossible de trouver le CSRF token dans la page de login".to_string())?;

    // Étape 2: Faire le POST de login
    let form_data = format!(
        "authenticity_token={}&user%5Bemail%5D={}&user%5Bpassword%5D={}&user%5Bremember_me%5D=0",
        encode(&csrf_token),
        encode(email),
        encode(password)
    );

    let login_response = match client
        .post("https://app.edsquare.fr/users/sign_in")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Origin", "https://app.edsquare.fr")
        .header("Referer", "https://app.edsquare.fr/")
        .body(form_data)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return Err(format!("Erreur lors de la connexion: {}", e));
        }
    };

    let login_status = login_response.status();
    let login_url = login_response.url().to_string();
    
    // Récupérer les cookies AVANT de consommer la réponse avec .text()
    let mut login_cookies: Vec<CookieItem> = Vec::new();
    for cookie in login_response.cookies() {
        login_cookies.push(CookieItem {
            name: cookie.name().to_string(),
            value: cookie.value().to_string(),
            domain: cookie.domain().map(|d| d.to_string()).unwrap_or_else(|| "app.edsquare.fr".to_string()),
            path: cookie.path().map(|p| p.to_string()).unwrap_or_else(|| "/".to_string()),
            expires: cookie.expires().and_then(|e| e.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64),
            http_only: cookie.http_only(),
            secure: cookie.secure(),
            same_site: None,
        });
    }
    
    // Vérifier si on est redirigé (succès) ou si on reste sur la page de login (échec)
    let is_redirected = login_status == StatusCode::FOUND 
        || login_status == StatusCode::SEE_OTHER 
        || login_url.contains("/home");
    
    // Si on n'est pas redirigé, vérifier le contenu pour détecter les erreurs
    if !is_redirected {
        let response_text = login_response.text().await.unwrap_or_default();
        // Vérifier les messages d'erreur courants
        if response_text.contains("Invalid email or password") 
            || response_text.contains("Email ou mot de passe invalide")
            || response_text.contains("incorrect")
            || response_text.contains("erreur")
            || (response_text.contains("sign_in") && !response_text.contains("/home")) {
            return Err("Échec de la connexion: identifiants invalides".to_string());
        }
        // Si on reste sur la page de login sans redirection, c'est un échec
        if login_url.contains("sign_in") && !login_url.contains("/home") {
            return Err("Échec de la connexion: identifiants invalides".to_string());
        }
    }

    // Étape 3: Faire une requête vers /home pour vérifier la connexion et récupérer les cookies
    let home_response = match client
        .get("https://app.edsquare.fr/home")
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return Err(format!("Erreur lors de la récupération de la page d'accueil: {}", e));
        }
    };

    // Vérifier que la requête vers /home a réussi (si on n'est pas connecté, on sera redirigé vers /users/sign_in)
    let home_status = home_response.status();
    let home_url = home_response.url().to_string();
    
    // Vérifier le header Location si présent (redirection)
    let location_header = home_response.headers()
        .get("location")
        .and_then(|loc| loc.to_str().ok())
        .map(|s| s.to_string());
    
    // Si on est redirigé vers sign_in, la connexion a échoué
    if home_url.contains("sign_in") {
        return Err("Échec de la connexion: identifiants invalides ou session expirée".to_string());
    }
    
    if let Some(ref location) = location_header {
        if location.contains("sign_in") {
            return Err("Échec de la connexion: identifiants invalides".to_string());
        }
    }
    
    // Vérifier que la connexion a vraiment réussi
    // Critères de succès :
    // 1. Le status doit être 200 (succès) OU une redirection valide
    // 2. L'URL finale ne doit PAS contenir "sign_in"
    // 3. Si redirection, le header Location ne doit PAS pointer vers "sign_in"
    
    let connection_successful = home_status.is_success() 
        && !home_url.contains("sign_in")
        && location_header.as_ref().map(|l| !l.contains("sign_in")).unwrap_or(true);
    
    if !connection_successful {
        warn!("Connexion EDSquare échouée: home_status={}, home_url={}, location={:?}", 
            home_status, home_url, location_header);
        return Err("Échec de la connexion: identifiants invalides".to_string());
    }
    
    info!("Connexion EDSquare réussie, vérification de /home OK (status: {}, url: {})", home_status, home_url);

    // Récupérer tous les cookies depuis les headers Set-Cookie
    let mut cookie_items: Vec<EdsquareCookieItem> = Vec::new();
    
    // Ajouter les cookies de la réponse de login (déjà récupérés)
    cookie_items.extend(login_cookies);
    
    // Récupérer les cookies de la réponse /home
    for cookie in home_response.cookies() {
        // Vérifier si le cookie n'existe pas déjà
        if !cookie_items.iter().any(|c| c.name == cookie.name()) {
            cookie_items.push(CookieItem {
                name: cookie.name().to_string(),
                value: cookie.value().to_string(),
                domain: cookie.domain().map(|d| d.to_string()).unwrap_or_else(|| "app.edsquare.fr".to_string()),
                path: cookie.path().map(|p| p.to_string()).unwrap_or_else(|| "/".to_string()),
                expires: cookie.expires().and_then(|e| e.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64),
                http_only: cookie.http_only(),
                secure: cookie.secure(),
                same_site: None,
            });
        }
    }
    
    // Vérifier qu'on a au moins un cookie de session important (comme _session_id ou similaire)
    if cookie_items.is_empty() {
        return Err("Aucun cookie reçu après la connexion. La connexion a peut-être échoué.".to_string());
    }

    // Vérifier qu'on a des cookies valides (au moins un cookie avec une valeur non vide)
    let has_valid_cookies = cookie_items.iter().any(|c| !c.value.is_empty() && c.value.len() > 10);
    if !has_valid_cookies {
        return Err("Les cookies reçus semblent invalides. La connexion a peut-être échoué.".to_string());
    }

    info!("Connexion EDSquare réussie, {} cookies à sauvegarder", cookie_items.len());
    
    // Afficher les noms des cookies pour le débogage
    let cookie_names: Vec<String> = cookie_items.iter().map(|c| c.name.clone()).collect();
    info!("Cookies reçus: {:?}", cookie_names);

    // Étape 4: Sauvegarder les cookies
    match save_edsquare_cookies(state, user_id_param, &cookie_items) {
        Ok(_) => {
            info!("Cookies EDSquare sauvegardés avec succès pour la date: {}", chrono::Utc::now().date_naive());
            // Sauvegarder également les identifiants pour permettre des reconnexions automatiques
            if let Err(e) = save_edsquare_credentials(state, user_id_param, email, password) {
                warn!("Connexion OK mais échec de la sauvegarde des identifiants EDSquare: {}", e);
            }
            Ok(LoginEdsquareResponse {
                success: true,
                message: "Connexion EDSquare réussie et cookies sauvegardés".to_string(),
            })
        },
        Err(e) => {
            error!("Erreur lors de la sauvegarde des cookies: {}", e);
            Err(format!("Connexion réussie mais erreur lors de la sauvegarde des cookies: {}", e))
        },
    }
}

/// Relance une connexion EDSquare en utilisant les identifiants sauvegardés en base.
pub async fn login_edsquare_with_saved(
    user_id_param: &str,
    state: &GlobalState,
) -> Result<LoginEdsquareResponse, String> {
    let creds = match get_edsquare_credentials(state, user_id_param) {
        Ok(Some(c)) => c,
        Ok(None) => {
            return Err("Aucun identifiant EDSquare enregistré pour cet utilisateur.".into());
        }
        Err(e) => {
            return Err(e);
        }
    };

    let (email, password) = creds;
    login_edsquare(&email, &password, user_id_param, state).await
}

fn extract_csrf_token_from_html(html: &str) -> Option<String> {
    // Chercher le token dans un input hidden avec name="authenticity_token"
    // Format: <input type="hidden" name="authenticity_token" value="TOKEN" />
    let pattern = r#"name="authenticity_token"\s+value="([^"]+)""#;
    let re = regex::Regex::new(pattern).ok()?;
    
    if let Some(captures) = re.captures(html) {
        return captures.get(1).map(|m| m.as_str().to_string());
    }

    // Alternative: chercher dans un meta tag
    let meta_pattern = r#"<meta\s+name="csrf-token"\s+content="([^"]+)""#;
    if let Ok(meta_re) = regex::Regex::new(meta_pattern) {
        if let Some(captures) = meta_re.captures(html) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
    }

    None
}
