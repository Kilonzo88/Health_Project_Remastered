pub mod auth;
pub mod did;
pub mod fhir;
pub mod hedera;
pub mod ipfs;
pub mod twilio;
pub mod gemini;
pub mod patient;
pub mod encounter;
pub mod vc;

pub use auth::{AuthService, AuthServiceImpl, RegistrationResponse, InitiateAuthResponse};
pub use patient::PatientService;
pub use encounter::EncounterService;
pub use vc::VerifiableCredentialService;
pub use gemini::ask_gemini;