use crate::config::load::Parameters;
use crate::handlers::common::get_error;
use crate::handlers::document::{Document, DocumentformInterface};
use custom_logger as log;
use serde_derive::{Deserialize, Serialize};
use std::fs;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
    pub usage_metadata: UsageMetadata,
    pub model_version: String,
    pub response_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: String,
    pub index: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    pub prompt_token_count: i64,
    pub candidates_token_count: i64,
    pub total_token_count: i64,
    pub prompt_tokens_details: Vec<PromptTokensDetail>,
    pub thoughts_token_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTokensDetail {
    pub modality: String,
    pub token_count: i64,
}

pub trait AgentInterface {
    async fn execute(params: Parameters, key: String)
    -> Result<String, Box<dyn std::error::Error>>;
}

pub struct Agent {}

impl AgentInterface for Agent {
    async fn execute(
        params: Parameters,
        key: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let db_path = params.db_path.clone();
        let fd = Document::get_formdata(format!("{}/queue", db_path.clone()), key.clone()).await?;
        log::debug!("[execute] gemini agent {:?}", fd);
        let prompt = fd.prompt;
        let data = match params.test {
            true => {
                log::info!("mode: test");
                fs::read("/home/lzuccarelli/Projects/rust-gemini-agent/docs/example-response.json")?
            }
            false => {
                log::info!("mode: execute");
                let gemini = fs::read_to_string("/home/lzuccarelli/.gemini/api-key")?;
                let gemini_url = format!("{}{}", params.base_url, gemini);
                log::debug!("[execute] url {}", gemini_url);
                let gemini_payload = get_gemini_payload(prompt);
                log::debug!("payload {}", gemini_payload);
                let client = reqwest::Client::new();
                let res = client.post(gemini_url).body(gemini_payload).send().await;
                match res {
                    Ok(data) => {
                        let data_result = data.bytes().await?;
                        data_result.to_vec()
                    }
                    Err(_) => {
                        vec![]
                    }
                }
            }
        };
        if data.len() > 0 {
            let gemini: GeminiResponse = serde_json::from_slice(&data)?;
            let gemini_document = gemini.candidates[0].content.parts[0].text.clone();
            log::info!("result from gemini\n\n {}", gemini_document);
            Document::save_formdata(db_path, key, gemini_document).await?;
            Ok("exit => 0".to_string())
        } else {
            Err(get_error(
                "no valid response from gemini inference server".to_string(),
            ))
        }
    }
}

fn get_gemini_payload(prompt: String) -> String {
    let payload = format!(
        r#"
{{
    "contents": [
      {{
        "role": "user",
        "parts": [
          {{
            "text": "{}"
          }},
        ]
      }},
    ],
    "generationConfig": {{
      "thinkingConfig": {{
        "thinkingBudget": -1,
      }},
      "responseMimeType": "text/plain",
    }},
}}    "#,
        prompt
    );
    payload
}
