use axum::{
    extract::{State, Request},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Validation, DecodingKey};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::state::AppState;
use crate::services::AuthService;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String, // Subject (user's DID)
    pub exp: usize,  // Expiration time
}

#[derive(Clone)]
pub struct AuthContext {
    pub user_did: String,
}


// Define the authentication middleware
pub async fn auth_middleware<T: AuthService>(
    State(state): State<Arc<AppState<T>>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        });

    let token = if let Some(token) = token {
        token
    } else {
        // No token provided
        return Err(StatusCode::UNAUTHORIZED);
    };

    let validation = Validation::default();
    let decoding_key = DecodingKey::from_secret(state.config.jwt_secret.as_ref());

    match decode::<AuthClaims>(&token, &decoding_key, &validation) {
        Ok(token_data) => {
            let auth_context = AuthContext {
                user_did: token_data.claims.sub,
            };
            req.extensions_mut().insert(auth_context);
            Ok(next.run(req).await)
        }
        Err(_) => {
            // Token is invalid
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}