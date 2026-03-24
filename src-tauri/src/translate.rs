use reqwest::Client;
use std::collections::HashMap;
use tokio::time::Instant;

const API_URL: &str = "https://translate.googleapis.com/translate_a/single";
const MIN_REQUEST_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

pub struct TranslationService {
    client: Client,
    cache: HashMap<String, String>,
    last_request_at: Option<Instant>,
}

impl TranslationService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: HashMap::new(),
            last_request_at: None,
        }
    }

    pub async fn translate(&mut self, text: &str) -> Result<String, String> {
        if let Some(cached) = self.cache.get(text) {
            return Ok(cached.clone());
        }

        self.wait_for_rate_limit().await;

        let response = self
            .client
            .get(API_URL)
            .query(&[
                ("client", "gtx"),
                ("sl", "en"),
                ("tl", "ja"),
                ("dt", "t"),
                ("q", text),
            ])
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        self.last_request_at = Some(Instant::now());

        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let translated = parse_google_response(&body)?;

        self.cache.insert(text.to_string(), translated.clone());

        Ok(translated)
    }

    async fn wait_for_rate_limit(&self) {
        if let Some(last) = self.last_request_at {
            let elapsed = last.elapsed();
            if elapsed < MIN_REQUEST_INTERVAL {
                tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
            }
        }
    }

}

/// Google Translate APIのレスポンスから翻訳テキストを抽出
/// レスポンス形式: [[["翻訳文","原文",...],...],...]
fn parse_google_response(body: &serde_json::Value) -> Result<String, String> {
    let sentences = body
        .get(0)
        .and_then(|v| v.as_array())
        .ok_or("Unexpected response format")?;

    let translated: String = sentences
        .iter()
        .filter_map(|sentence| sentence.get(0).and_then(|v| v.as_str()))
        .collect();

    if translated.is_empty() {
        return Err("No translation found in response".to_string());
    }

    Ok(translated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_service_has_empty_cache() {
        let service = TranslationService::new();
        assert_eq!(service.cache.len(), 0);
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let mut service = TranslationService::new();
        service
            .cache
            .insert("hello".to_string(), "こんにちは".to_string());

        let result = service.translate("hello").await;
        assert_eq!(result.unwrap(), "こんにちは");
        assert_eq!(service.cache.len(), 1);
    }

    #[tokio::test]
    async fn test_cache_hit_skips_api_call() {
        let mut service = TranslationService::new();
        service
            .cache
            .insert("hello".to_string(), "こんにちは".to_string());

        let _ = service.translate("hello").await;
        // キャッシュヒット時はlast_request_atが更新されない
        assert!(service.last_request_at.is_none());
    }

    #[test]
    fn test_parse_google_response_single_sentence() {
        let body = serde_json::json!([
            [["こんにちは", "hello", null, null, 10]],
            null,
            "en"
        ]);
        let result = parse_google_response(&body).unwrap();
        assert_eq!(result, "こんにちは");
    }

    #[test]
    fn test_parse_google_response_multiple_sentences() {
        let body = serde_json::json!([
            [
                ["こんにちは。", "Hello.", null, null, 10],
                ["世界。", "World.", null, null, 10]
            ],
            null,
            "en"
        ]);
        let result = parse_google_response(&body).unwrap();
        assert_eq!(result, "こんにちは。世界。");
    }

    #[test]
    fn test_parse_google_response_invalid_format() {
        let body = serde_json::json!({"error": "bad request"});
        let result = parse_google_response(&body);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_google_response_empty_sentences() {
        let body = serde_json::json!([[]]);
        let result = parse_google_response(&body);
        assert!(result.is_err());
    }
}
