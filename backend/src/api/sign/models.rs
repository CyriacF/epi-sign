use chrono::NaiveDate;
use diesel::{Selectable, pg::Pg, prelude::Queryable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ulid::Ulid;
use utoipa::ToSchema;

use crate::schema::cookies;

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = cookies)]
#[diesel(check_for_backend(Pg))]
pub struct Cookie {
    pub id: String,
    pub date: NaiveDate,
    pub cookie_data: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CookieItem {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<i64>,
    #[serde(rename = "httpOnly")]
    pub http_only: bool,
    pub secure: bool,
    #[serde(rename = "sameSite")]
    pub same_site: Option<String>,
}

impl CookieItem {
    pub fn to_string(&self) -> String {
        format!(
            "{}={}; Domain={}; Path={}; Expires={}; HttpOnly={}; Secure={}; SameSite={:?}",
            self.name,
            self.value,
            self.domain,
            self.path,
            self.expires.map_or("None".to_string(), |e| e.to_string()),
            self.http_only,
            self.secure,
            self.same_site
        )
    }

    pub fn to_header_value(&self) -> String {
        format!("{}={}", self.name, self.value)
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignPayload {
    #[schema(value_type = Vec<String>, example = "[\"01F8MECHZX3TBDSZ7X4F5G9Z6H\", \"01F8MECHZX3TBDSZ7X4F5G9Z6I\"]")]
    pub ulids: Vec<Ulid>,
    #[schema(value_type = String, example = "https://intra.epitech.eu/module/XXXX/X-XXX-000/XXX-0-0/acti-000000/event-000000/registered?token=00000000")]
    pub url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserSignResponse {
    #[schema(value_type = String, example = "01F8MECHZX3TBDSZ7X4F5G9Z6H")]
    pub ulid: String,
    pub response: SignResponse,
}

#[derive(Debug, Serialize, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SignResponse {
    Success,
    TokenExpired,
    TokenNotFound,
    AlreadySigned,
    UnknownError,
    ServiceUnavailable,
    // BadToken,
}
