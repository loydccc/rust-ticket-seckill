use axum::{async_trait, extract::FromRequestParts, http::{request::Parts, StatusCode}};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub uid: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub username: String,
}

pub fn jwt_secret() -> String {
    std::env::var("DEV_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
}

pub fn issue_token(user_id: Uuid, username: &str) -> anyhow::Result<String> {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: username.to_string(),
        uid: user_id.to_string(),
        exp,
    };
    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )?;
    Ok(token)
}

pub fn decode_token(token: &str) -> Result<AuthUser, AppError> {
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let user_id = Uuid::parse_str(&data.claims.uid).map_err(|_| AppError::Unauthorized)?;
    Ok(AuthUser {
        user_id,
        username: data.claims.sub,
    })
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = match auth.strip_prefix("Bearer ") {
            Some(t) if !t.is_empty() => t,
            _ => return Err(AppError::Unauthorized.into_response()),
        };

        decode_token(token).map_err(|e| e.into_response())
    }
}
