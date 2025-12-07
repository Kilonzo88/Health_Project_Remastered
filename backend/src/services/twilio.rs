use twilio::{Client, OutboundMessage};
use crate::config::Config;
use anyhow::anyhow;

pub struct TwilioService {
    client: Client,
    from_phone_number: String,
}

impl TwilioService {
    pub fn new(config: &Config) -> Self {
        let client = Client::new(&config.twilio_account_sid, &config.twilio_auth_token);
        let from_phone_number = config.twilio_phone_number.clone();
        Self { client, from_phone_number }
    }

    pub fn send_otp(&self, to: &str, otp: &str) -> anyhow::Result<()> {
        let body = format!("Your OTP is: {}", otp);
        let message = OutboundMessage::new(&self.from_phone_number, to, &body);
        self.client.send_message(message).map_err(|e| anyhow!("{:?}", e))?;
        Ok(())
    }
}
