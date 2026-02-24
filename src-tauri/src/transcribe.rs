use reqwest::multipart;

pub async fn transcribe_whisper(
    api_key: &str,
    wav_data: Vec<u8>,
    language: &str,
    glossary_prompt: Option<&str>,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let file_part = multipart::Part::bytes(wav_data)
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Mime error: {e}"))?;

    let mut form = multipart::Form::new()
        .text("model", "whisper-1")
        .text("response_format", "text")
        .text("language", language.to_string())
        .part("file", file_part);

    if let Some(prompt) = glossary_prompt {
        if !prompt.is_empty() {
            form = form.text("prompt", prompt.to_string());
        }
    }

    let resp = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Request error: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Whisper API error ({status}): {body}"));
    }

    resp.text()
        .await
        .map(|t| t.trim().to_string())
        .map_err(|e| format!("Response read error: {e}"))
}
