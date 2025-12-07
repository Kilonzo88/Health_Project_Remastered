
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::config::Config;

// --- Gemini API Structs ---
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "safetySettings")]
    safety_settings: Vec<SafetySetting>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct SafetySetting {
    category: String,
    threshold: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Deserialize)]
struct PartResponse {
    text: String,
}

pub async fn ask_gemini(prompt: &str, config: &Config) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let api_key = &config.gemini_api_key;
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}", api_key);

    let request_body = GeminiRequest {
        contents: vec![Content { parts: vec![Part { text: prompt.to_string() }] }],
        safety_settings: vec![
            SafetySetting {
                category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                threshold: "BLOCK_ONLY_HIGH".to_string(),
            }
        ]
    };

    let res = client.post(&url)
        .json(&request_body)
        .send()
        .await?;

    if res.status().is_success() {
        let gemini_response = res.json::<GeminiResponse>().await?;
        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
        Err(anyhow!("No content found in Gemini response"))
    } else {
        let error_body = res.text().await?;
        Err(anyhow!("Gemini API request failed: {}", error_body))
    }
}
