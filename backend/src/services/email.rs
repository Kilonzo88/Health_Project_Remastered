use crate::config::SmtpConfig;
use lazy_static::lazy_static;
use lettre::{
    message::{header, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::Serialize;
use std::sync::Arc;
use tera::{Context, Tera};
use thiserror::Error;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("backend/src/templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        tera
    };
}

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Template error: {0}")]
    TemplateError(#[from] tera::Error),
    #[error("Invalid email address: {0}")]
    InvalidAddress(#[from] lettre::address::AddressError),
    #[error("Failed to build email: {0}")]
    EmailBuild(#[from] lettre::error::Error),
    #[error("SMTP transport error: {0}")]
    SmtpTransport(#[from] lettre::transport::smtp::Error),
}

#[derive(Serialize)]
pub struct WelcomeEmailContext {
    pub username: String,
}

#[derive(Serialize)]
pub struct VerificationEmailContext {
    pub username: String,
    pub verification_link: String,
}

#[derive(Clone)]
pub struct EmailService {
    smtp_config: Arc<SmtpConfig>,
}

impl EmailService {
    pub fn new(smtp_config: Arc<SmtpConfig>) -> Self {
        Self { smtp_config }
    }

    async fn send_mail<T: Serialize>(
        &self,
        to_email: &str,
        subject: &str,
        template_name: &str,
        context: &T,
    ) -> Result<(), EmailError> {
        let context = Context::from_serialize(context)?;
        let html_template = TEMPLATES.render(template_name, &context)?;

        let email = Message::builder()
            .from(self.smtp_config.from_email.parse()?)
            .to(to_email.parse()?)
            .subject(subject)
            .header(header::ContentType::TEXT_HTML)
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(html_template),
            )?;

        let creds = Credentials::new(
            self.smtp_config.username.clone(),
            self.smtp_config.password.clone(),
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.smtp_config.server)?
            .credentials(creds)
            .port(self.smtp_config.port)
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build();

        mailer.send(email).await?;

        Ok(())
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        username: &str,
        token: &str,
        base_url: &str,
    ) -> Result<(), EmailError> {
        let subject = "Email Verification";
        let template_name = "Verification-email.html";
        // Note: The user mentioned FlutterFlow, the verification link might need to be a deep link.
        // For now, creating a standard backend verification link.
        let verification_link = format!("{}/api/auth/verify?token={}", base_url, token);

        let context = VerificationEmailContext {
            username: username.to_string(),
            verification_link,
        };

        self.send_mail(to_email, subject, template_name, &context).await
    }

    pub async fn send_welcome_email(
        &self,
        to_email: &str,
        username: &str,
    ) -> Result<(), EmailError> {
        let subject = "Welcome to Our Application";
        let template_name = "Welcome-email.html";

        let context = WelcomeEmailContext {
            username: username.to_string(),
        };

        self.send_mail(to_email, subject, template_name, &context).await
    }
}
