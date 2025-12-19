use crate::config::Config;
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
    config: Arc<Config>,
}

impl EmailService {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
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
            .from(self.config.smtp.from_email.parse()?)
            .to(to_email.parse()?)
            .subject(subject)
            .header(header::ContentType::TEXT_HTML)
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(html_template),
            )?;

        let creds = Credentials::new(
            self.config.smtp.username.clone(),
            self.config.smtp.password.clone(),
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.smtp.server)?
            .credentials(creds)
            .port(self.config.smtp.port)
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build();

        mailer.send(email).await?;

        Ok(())
    }

    pub fn send_verification_email(
        &self,
        to_email: &str,
        username: &str,
        token: &str,
    ) {
        let subject = "Email Verification";
        let template_name = "Verification-email.html";
        // Point verification link to FlutterFlow app
        // FlutterFlow will handle the UI and call the backend API
        let verification_link = format!("{}/verify-email?token={}", self.config.frontend_base_url.trim_end_matches('/'), token);

        let context = VerificationEmailContext {
            username: username.to_string(),
            verification_link,
        };

        let email_service = self.clone();
        let to_email = to_email.to_string();
        let subject = subject.to_string();
        let template_name = template_name.to_string();
        tokio::spawn(async move {
            tracing::info!("Sending verification email to {}", to_email);
            if let Err(e) = email_service.send_mail(&to_email, &subject, &template_name, &context).await {
                tracing::error!("Failed to send verification email to {}: {}", to_email, e);
            }
        });
    }

    pub fn send_welcome_email(
        &self,
        to_email: &str,
        username: &str,
    ) {
        let subject = "Welcome to Our Application";
        let template_name = "Welcome-email.html";

        let context = WelcomeEmailContext {
            username: username.to_string(),
        };

        let email_service = self.clone();
        let to_email = to_email.to_string();
        tokio::spawn(async move {
            tracing::info!("Sending welcome email to {}", to_email);
            if let Err(e) = email_service.send_mail(&to_email, subject, template_name, &context).await {
                tracing::error!("Failed to send welcome email to {}: {}", to_email, e);
            }
        });
    }
}
