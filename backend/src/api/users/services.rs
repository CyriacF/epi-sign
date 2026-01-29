use axum::{
    Router,
    routing::{get, patch, post},
};
use chrono::NaiveDateTime;
use ulid::Ulid;

use super::models::User;
use crate::{api::auth::RegisterPayload, misc::GlobalState};

pub fn get_routes(state: GlobalState) -> Router {
    Router::new()
        .route("/", get(super::endpoints::get_users))
        .route("/me", get(super::endpoints::get_me))
        .route("/me", patch(super::endpoints::update_user))
        .route("/me/update-jwt", post(super::endpoints::update_jwt))
        .route("/me/signature", post(super::endpoints::save_signature))
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

pub fn save_user_signature(state: &GlobalState, user: &User) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.get_db_conn() {
        Ok(conn) => conn,
        Err(_) => return Err(diesel::result::Error::NotFound),
    };

    diesel::update(users.filter(id.eq(&user.id)))
        .set(signature_manuscrite.eq(&user.signature_manuscrite))
        .execute(&mut conn)?;

    Ok(())
}
