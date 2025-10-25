use axum::{
    extract::{State, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::state::AppState;
use crate::auth::AuthContext;

// Define the high-assurance authentication middleware
pub async fn high_assurance_auth_middleware(State(state): State<Arc<AppState>>, mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_context = req.extensions().get::<AuthContext>().cloned();

    if let Some(auth_context) = auth_context {
        // In a real implementation, we would check for a high-assurance session.
        // For now, we'll just check if the user is authenticated.
        // This can be extended to check for a recent login, a second factor, etc.
        if !auth_context.user_did.is_empty() {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
