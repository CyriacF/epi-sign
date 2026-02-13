use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use rand::seq::SliceRandom;
use ulid::Ulid;

use super::models::{User, UserSignature};
use crate::{api::auth::RegisterPayload, misc::GlobalState};

pub fn get_routes(state: GlobalState) -> Router {
    Router::new()
        .route("/", get(super::endpoints::get_users))
        .route("/me", get(super::endpoints::get_me))
        .route("/me", patch(super::endpoints::update_user))
        .route("/me", delete(super::endpoints::delete_account))
        .route("/me/update-jwt", post(super::endpoints::update_jwt))
        .route("/me/signature", post(super::endpoints::save_signature))
        .route("/me/signatures", get(super::endpoints::get_signatures))
        .route("/me/signatures/{id}", delete(super::endpoints::delete_signature))
        .with_state(state)
}

pub fn create_user(state: &GlobalState, user: User) -> Result<(), diesel::result::Error> {
    use crate::schema::users;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    diesel::insert_into(users::table)
        .values(&user)
        .execute(&mut conn)?;

    Ok(())
}

pub fn user_exists(
    state: &GlobalState,
    user: &RegisterPayload,
) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    let user_exists = users
        .filter(username.eq(&user.username))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .optional()?;

    if user_exists.is_some() {
        return Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            Box::new("User already exists".to_string()),
        ));
    }

    Ok(())
}

pub fn get_user_by_username(
    state: &GlobalState,
    query_username: &str,
) -> Result<Option<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    users
        .filter(username.eq(query_username))
        .select(User::as_select())
        .first(&mut conn)
        .optional()
}

pub fn get_user_by_id(
    state: &GlobalState,
    user_id: &Ulid,
) -> Result<Option<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    users
        .filter(id.eq(user_id.to_string()))
        .select(User::as_select())
        .first(&mut conn)
        .optional()
}

pub fn update_user_jwt(
    state: &GlobalState,
    user_id: Ulid,
    new_jwt: &str,
    new_jwt_exp: NaiveDateTime,
) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    diesel::update(users.filter(id.eq(user_id.to_string())))
        .set((
            jwt_intra_epitech.eq(new_jwt),
            jwt_expires_at.eq(new_jwt_exp),
        ))
        .execute(&mut conn)?;

    Ok(())
}

pub fn get_users_by_ulids(
    state: &GlobalState,
    user_ids: &Vec<Ulid>,
) -> Result<Vec<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    let user_id_strings: Vec<String> = user_ids
        .iter()
        .map(|other_id| other_id.to_string())
        .collect();

    users
        .filter(id.eq_any(user_id_strings))
        .select(User::as_select())
        .load(&mut conn)
}

pub fn get_all_users(state: &GlobalState) -> Result<Vec<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    users.select(User::as_select()).load(&mut conn)
}

pub fn update_user(state: &GlobalState, user: &User) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    diesel::update(users.filter(id.eq(&user.id)))
        .set((
            username.eq(&user.username),
            password_hash.eq(&user.password_hash),
            jwt_intra_epitech.eq(&user.jwt_intra_epitech),
            jwt_expires_at.eq(&user.jwt_expires_at),
            signature_manuscrite.eq(&user.signature_manuscrite),
        ))
        .execute(&mut conn)?;

    Ok(())
}

pub fn add_user_signature(
    state: &GlobalState,
    user_id_param: &str,
    signature_data_param: &str,
) -> Result<UserSignature, diesel::result::Error> {
    use crate::schema::user_signatures::dsl::*;

    let mut conn = state.get_db_conn().map_err(|_| diesel::result::Error::NotFound)?;
    let sig_id = Ulid::new().to_string();
    let now = Utc::now().naive_utc();
    diesel::insert_into(user_signatures)
        .values((
            id.eq(&sig_id),
            user_id.eq(user_id_param),
            signature_data.eq(signature_data_param),
            created_at.eq(now),
        ))
        .execute(&mut conn)?;
    Ok(UserSignature {
        id: sig_id,
        user_id: user_id_param.to_string(),
        signature_data: signature_data_param.to_string(),
        created_at: now,
    })
}

pub fn get_user_signatures(
    state: &GlobalState,
    user_id_param: &str,
) -> Result<Vec<UserSignature>, diesel::result::Error> {
    use crate::schema::user_signatures::dsl::*;

    let mut conn = state.get_db_conn().map_err(|_| diesel::result::Error::NotFound)?;
    user_signatures
        .filter(user_id.eq(user_id_param))
        .order(created_at.desc())
        .select(UserSignature::as_select())
        .load(&mut conn)
}

pub fn get_random_signature_for_user(
    state: &GlobalState,
    user_id_param: &str,
) -> Result<Option<String>, diesel::result::Error> {
    let sigs = get_user_signatures(state, user_id_param)?;
    if sigs.is_empty() {
        return Ok(None);
    }
    let mut rng = rand::thread_rng();
    let chosen = sigs.choose(&mut rng).map(|s| s.signature_data.clone());
    Ok(chosen)
}

pub fn delete_user_signature(
    state: &GlobalState,
    signature_id: &str,
    user_id_param: &str,
) -> Result<bool, diesel::result::Error> {
    use crate::schema::user_signatures::dsl::*;

    let mut conn = state.get_db_conn().map_err(|_| diesel::result::Error::NotFound)?;
    let deleted = diesel::delete(user_signatures)
        .filter(id.eq(signature_id))
        .filter(user_id.eq(user_id_param))
        .execute(&mut conn)?;
    Ok(deleted > 0)
}

/// Supprime le compte utilisateur et toutes les données associées (signatures, EDSquare cookies/credentials).
pub fn delete_user_account(
    state: &GlobalState,
    user_id_param: &str,
) -> Result<bool, diesel::result::Error> {
    let mut conn = state.get_db_conn().map_err(|_| diesel::result::Error::NotFound)?;

    {
        use crate::schema::user_signatures::dsl::*;
        diesel::delete(user_signatures.filter(user_id.eq(user_id_param))).execute(&mut conn)?;
    }
    {
        use crate::schema::edsquare_credentials::dsl::*;
        diesel::delete(edsquare_credentials.filter(user_id.eq(user_id_param))).execute(&mut conn)?;
    }
    {
        use crate::schema::edsquare_cookies::dsl::*;
        diesel::delete(edsquare_cookies.filter(user_id.eq(user_id_param))).execute(&mut conn)?;
    }
    {
        use crate::schema::users::dsl::*;
        let deleted = diesel::delete(users.filter(id.eq(user_id_param))).execute(&mut conn)?;
        return Ok(deleted > 0);
    }
}
