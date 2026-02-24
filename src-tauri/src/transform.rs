use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

pub async fn transform_text(
    api_key: &str,
    api_url: &str,
    model: &str,
    prompt_template: &str,
    transcribed_text: &str,
) -> Result<String, String> {
    let system_prompt = prompt_template.replace("{text}", transcribed_text);

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are a helpful text transformation assistant. Output ONLY the transformed text with no additional commentary.".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: system_prompt,
        },
    ];

    let body = ChatRequest {
        model: model.to_string(),
        messages,
        temperature: 0.3,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(api_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request error: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("LLM API error ({status}): {body}"));
    }

    let chat_resp: ChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {e}"))?;

    chat_resp
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "No response from LLM".to_string())
}
