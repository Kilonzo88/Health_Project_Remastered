use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`
use std::sync::Arc;
use health_remastered::services::MockAuthService;
use health_remastered::models::{Patient, FhirPatient};
use health_remastered::services::RegistrationResponse;

#[tokio::test]
async fn test_auth_google() {
    let mut mock_auth_service = MockAuthService::new();

    mock_auth_service.expect_authenticate_with_google()
        .returning(|_| Ok(RegistrationResponse {
            user: Patient {
                id: None,
                did: "did:hedera:testnet:123".to_string(),
                fhir_patient: FhirPatient::default(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            token: "dummy_token".to_string(),
        }));

    let app = create_app(Arc::new(mock_auth_service)).await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/google")
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{
                "id_token": "dummy_token"
            }"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
