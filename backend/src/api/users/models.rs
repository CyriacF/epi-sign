use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::api::auth::hash_password;

use crate::api::auth::RegisterPayload;
use crate::api::sign::CookieItem;

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable, ToSchema)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[schema(example = "01F8MECHZX3TBDSZ7XK4F5G9ZQ")]
    pub id: String,
    #[schema(example = "johndoe")]
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,

    #[schema(
        example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.KMUFsIDTnFmyG3nMiGM6H9FNFUROf3wh7SmqJp-QV30"
    )]
    pub jwt_intra_epitech: Option<String>,
    pub jwt_expires_at: Option<chrono::NaiveDateTime>,
    #[schema(example = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...")]
    pub signature_manuscrite: Option<String>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self {
            id: Ulid::new().to_string(),
            username,
            password_hash: hash_password(password.as_str()),
            jwt_intra_epitech: None,
            jwt_expires_at: None,
            signature_manuscrite: None,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        hash_password(password) == self.password_hash
    }

    pub fn get_jwt_as_cookie(&self) -> Option<CookieItem> {
        let jwt = match &self.jwt_intra_epitech {
            Some(jwt) => jwt,
            None => return None,
        };

        Some(CookieItem {
            name: "user".to_string(),
            value: jwt.clone(),
            domain: "intra.epitech.eu".to_string(),
            path: "/".to_string(),
            expires: self.jwt_expires_at.map(|dt| dt.and_utc().timestamp()),
            http_only: true,
            secure: true,
            same_site: None,
        })
    }
}

impl From<RegisterPayload> for User {
    fn from(payload: RegisterPayload) -> Self {
        Self::new(payload.username, payload.password)
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct JwtPayload {
    #[schema(
        example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.KMUFsIDTnFmyG3nMiGM6H9FNFUROf3wh7SmqJp-QV30"
    )]
    pub jwt: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicUserResponse {
    #[schema(example = "01F8MECHZX3TBDSZ7XK4F5G9ZQ")]
    pub id: String,
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "2023-10-01T12:00:00")]
    pub jwt_expires_at: Option<chrono::NaiveDateTime>,
}

impl From<User> for PublicUserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            jwt_expires_at: user.jwt_expires_at,
        }
    }
}
#[derive(Deserialize, ToSchema, Debug)]
pub struct UpdateUserPayload {
    pub username: Option<String>,
    pub old_password: Option<String>,
    pub new_password: Option<String>,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct SaveSignaturePayload {
    #[schema(example = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...")]
    pub signature: String,
}

