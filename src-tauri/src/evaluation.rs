use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::backtrace::Backtrace;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalConfig {
    pub enabled: bool,
    pub sampling_rate: u32,
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_ms: u64,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 1,
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: None,
            timeout_ms: 8_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationInput {
    pub event_id: String,
    pub event_type: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredEvaluation {
    pub risk_level: String,
    pub risk_category: String,
    pub efficiency_level: String,
    pub suggestion: String,
}

trait Provider: Send + Sync {
    fn evaluate(
        &self,
        input: &EvaluationInput,
        config: &EvalConfig,
    ) -> Result<StructuredEvaluation, String>;
}

struct OpenAiProvider;
struct AnthropicProvider;
struct OllamaProvider;

impl Provider for OpenAiProvider {
    fn evaluate(&self, input: &EvaluationInput, config: &EvalConfig) -> Result<StructuredEvaluation, String> {
        let api_key = config.api_key.as_deref().unwrap_or("").trim();
        if api_key.is_empty() {
            return Err(classified_error(
                "config",
                "openai api_key is empty",
                &input.event_id,
                None,
            ));
        }

        let endpoint = resolve_endpoint(&config.base_url, "https://api.openai.com/v1", "/chat/completions");
        let client = build_client(config.timeout_ms, &input.event_id)?;
        let body = json!({
            "model": config.model,
            "temperature": 0.2,
            "response_format": { "type": "json_object" },
            "messages": [
                {
                    "role": "system",
                    "content": "你是代码行为评估器。只返回 JSON，字段必须包含 risk_level,risk_category,efficiency_level,suggestion。"
                },
                {
                    "role": "user",
                    "content": build_prompt(input)
                }
            ]
        });

        let response = client
            .post(endpoint)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .map_err(|error| classify_reqwest_error("openai", error, &input.event_id))?;

        parse_openai_response(response, &input.event_id)
    }
}

impl Provider for AnthropicProvider {
    fn evaluate(&self, input: &EvaluationInput, config: &EvalConfig) -> Result<StructuredEvaluation, String> {
        let api_key = config.api_key.as_deref().unwrap_or("").trim();
        if api_key.is_empty() {
            return Err(classified_error(
                "config",
                "anthropic api_key is empty",
                &input.event_id,
                None,
            ));
        }

        let endpoint = resolve_endpoint(&config.base_url, "https://api.anthropic.com/v1", "/messages");
        let client = build_client(config.timeout_ms, &input.event_id)?;
        let body = json!({
            "model": config.model,
            "max_tokens": 600,
            "system": "你是代码行为评估器。只返回 JSON，字段必须包含 risk_level,risk_category,efficiency_level,suggestion。",
            "messages": [
                {
                    "role": "user",
                    "content": build_prompt(input)
                }
            ]
        });

        let response = client
            .post(endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .map_err(|error| classify_reqwest_error("anthropic", error, &input.event_id))?;

        parse_anthropic_response(response, &input.event_id)
    }
}

impl Provider for OllamaProvider {
    fn evaluate(&self, input: &EvaluationInput, config: &EvalConfig) -> Result<StructuredEvaluation, String> {
        let endpoint = resolve_endpoint(&config.base_url, "http://127.0.0.1:11434/api", "/chat");
        let client = build_client(config.timeout_ms, &input.event_id)?;
        let body = json!({
            "model": config.model,
            "stream": false,
            "format": "json",
            "messages": [
                {
                    "role": "system",
                    "content": "你是代码行为评估器。只返回 JSON，字段必须包含 risk_level,risk_category,efficiency_level,suggestion。"
                },
                {
                    "role": "user",
                    "content": build_prompt(input)
                }
            ]
        });

        let response = client
            .post(endpoint)
            .json(&body)
            .send()
            .map_err(|error| classify_reqwest_error("ollama", error, &input.event_id))?;

        parse_ollama_response(response, &input.event_id)
    }
}

static FAIL_ONCE_SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn should_fail_once(event_id: &str) -> bool {
    let seen = FAIL_ONCE_SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    let mut guard = seen.lock().expect("fail-once set lock should be available");
    if guard.contains(event_id) {
        false
    } else {
        guard.insert(event_id.to_string());
        true
    }
}

fn resolve_endpoint(base_url: &str, default_base: &str, path: &str) -> String {
    let base = if base_url.trim().is_empty() {
        default_base.to_string()
    } else {
        base_url.trim().to_string()
    };

    if base.ends_with(path) {
        return base;
    }
    format!("{}{}", base.trim_end_matches('/'), path)
}

fn build_client(timeout_ms: u64, event_id: &str) -> Result<Client, String> {
    let timeout = timeout_ms.clamp(500, 120_000);
    Client::builder()
        .timeout(Duration::from_millis(timeout))
        .build()
        .map_err(|error| {
            classified_error(
                "config",
                &format!("failed to build http client: {error:?}"),
                event_id,
                None,
            )
        })
}

fn classify_reqwest_error(provider: &str, error: reqwest::Error, event_id: &str) -> String {
    let category = if error.is_timeout() { "network_timeout" } else { "network" };
    classified_error(
        category,
        &format!("provider={provider} request failed: {error:?}"),
        event_id,
        None,
    )
}

fn parse_openai_response(response: reqwest::blocking::Response, event_id: &str) -> Result<StructuredEvaluation, String> {
    let status = response.status();
    let text = response.text().unwrap_or_default();
    if !status.is_success() {
        return Err(classify_http_status("openai", status, &text, event_id));
    }

    let value: Value = serde_json::from_str(&text).map_err(|error| {
        classified_error(
            "parse",
            &format!("openai response json parse failed: {error:?}; body={text}"),
            event_id,
            None,
        )
    })?;
    let content = value
        .get("choices")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("message"))
        .and_then(|v| v.get("content"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            classified_error(
                "parse",
                &format!("openai response missing message.content; body={text}"),
                event_id,
                None,
            )
        })?;

    parse_structured_output(content, event_id)
}

fn parse_anthropic_response(
    response: reqwest::blocking::Response,
    event_id: &str,
) -> Result<StructuredEvaluation, String> {
    let status = response.status();
    let text = response.text().unwrap_or_default();
    if !status.is_success() {
        return Err(classify_http_status("anthropic", status, &text, event_id));
    }

    let value: Value = serde_json::from_str(&text).map_err(|error| {
        classified_error(
            "parse",
            &format!("anthropic response json parse failed: {error:?}; body={text}"),
            event_id,
            None,
        )
    })?;
    let content = value
        .get("content")
        .and_then(|v| v.as_array())
        .and_then(|items| items.first())
        .and_then(|item| item.get("text"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            classified_error(
                "parse",
                &format!("anthropic response missing content[0].text; body={text}"),
                event_id,
                None,
            )
        })?;

    parse_structured_output(content, event_id)
}

fn parse_ollama_response(response: reqwest::blocking::Response, event_id: &str) -> Result<StructuredEvaluation, String> {
    let status = response.status();
    let text = response.text().unwrap_or_default();
    if !status.is_success() {
        return Err(classify_http_status("ollama", status, &text, event_id));
    }

    let value: Value = serde_json::from_str(&text).map_err(|error| {
        classified_error(
            "parse",
            &format!("ollama response json parse failed: {error:?}; body={text}"),
            event_id,
            None,
        )
    })?;
    let content = value
        .get("message")
        .and_then(|v| v.get("content"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            classified_error(
                "parse",
                &format!("ollama response missing message.content; body={text}"),
                event_id,
                None,
            )
        })?;

    parse_structured_output(content, event_id)
}

fn classify_http_status(provider: &str, status: StatusCode, body: &str, event_id: &str) -> String {
    let category = match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "auth",
        StatusCode::TOO_MANY_REQUESTS => "rate_limit",
        _ => "provider",
    };
    classified_error(
        category,
        &format!("provider={provider} status={} body={body}", status.as_u16()),
        event_id,
        Some(status.as_u16()),
    )
}

fn build_prompt(input: &EvaluationInput) -> String {
    let mut sanitized_payload = input.payload.clone();
    crate::collection::sanitize_json_value(&mut sanitized_payload);
    format!(
        "请评估以下事件并输出 JSON:\n\
         event_id: {}\n\
         event_type: {}\n\
         payload: {}\n\
         仅输出 JSON。",
        input.event_id, input.event_type, sanitized_payload
    )
}

fn parse_structured_output(content: &str, event_id: &str) -> Result<StructuredEvaluation, String> {
    if let Ok(direct) = serde_json::from_str::<StructuredEvaluation>(content) {
        return Ok(direct);
    }

    if let Some(json_slice) = extract_json_object(content) {
        if let Ok(value) = serde_json::from_str::<StructuredEvaluation>(json_slice) {
            return Ok(value);
        }
    }

    Err(classified_error(
        "parse",
        &format!("model output is not valid StructuredEvaluation json: {content}"),
        event_id,
        None,
    ))
}

fn extract_json_object(content: &str) -> Option<&str> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&content[start..=end])
}

fn classified_error(category: &str, message: &str, event_id: &str, status: Option<u16>) -> String {
    let status_text = status
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    format!(
        "error_type={category} status={status_text} event_id={event_id} message={message} backtrace={:?}",
        Backtrace::capture()
    )
}

pub fn evaluate(input: &EvaluationInput, config: &EvalConfig) -> Result<StructuredEvaluation, String> {
    if input
        .payload
        .get("force_error")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Err(classified_error(
            "provider",
            "simulated permanent failure by force_error",
            &input.event_id,
            None,
        ));
    }

    if input
        .payload
        .get("force_error_once")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
        && should_fail_once(&input.event_id)
    {
        return Err(classified_error(
            "network",
            "simulated transient failure by force_error_once",
            &input.event_id,
            None,
        ));
    }

    match config.provider.as_str() {
        "openai" => OpenAiProvider.evaluate(input, config),
        "anthropic" => AnthropicProvider.evaluate(input, config),
        "ollama" => OllamaProvider.evaluate(input, config),
        other => Err(classified_error(
            "config",
            &format!("unsupported provider={other}"),
            &input.event_id,
            None,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn openai_provider_successfully_parses_structured_json() {
        let body = r#"{"choices":[{"message":{"content":"{\"risk_level\":\"low\",\"risk_category\":\"normal\",\"efficiency_level\":\"medium\",\"suggestion\":\"ok\"}"}}]}"#;
        let (base_url, _join) = spawn_mock_http_server(200, body.to_string(), 0);
        let config = EvalConfig {
            provider: "openai".to_string(),
            model: "gpt-test".to_string(),
            base_url,
            api_key: Some("sk-test".to_string()),
            timeout_ms: 2000,
            ..EvalConfig::default()
        };
        let input = EvaluationInput {
            event_id: "evt-mock-success".to_string(),
            event_type: "tool.call".to_string(),
            payload: json!({"cmd":"echo hi"}),
        };

        let result = evaluate(&input, &config).expect("openai mock response should parse");
        assert_eq!(result.risk_level, "low");
        assert_eq!(result.risk_category, "normal");
    }

    #[test]
    fn openai_provider_classifies_auth_error() {
        let (base_url, _join) = spawn_mock_http_server(401, "{\"error\":\"unauthorized\"}".to_string(), 0);
        let config = EvalConfig {
            provider: "openai".to_string(),
            model: "gpt-test".to_string(),
            base_url,
            api_key: Some("invalid".to_string()),
            timeout_ms: 2000,
            ..EvalConfig::default()
        };
        let input = EvaluationInput {
            event_id: "evt-auth-fail".to_string(),
            event_type: "tool.call".to_string(),
            payload: json!({}),
        };

        let error = evaluate(&input, &config).expect_err("auth response should fail");
        assert!(error.contains("error_type=auth"));
    }

    #[test]
    fn openai_provider_classifies_timeout_error() {
        let (base_url, _join) = spawn_mock_http_server(
            200,
            "{\"choices\":[{\"message\":{\"content\":\"{}\"}}]}".to_string(),
            1200,
        );
        let config = EvalConfig {
            provider: "openai".to_string(),
            model: "gpt-test".to_string(),
            base_url,
            api_key: Some("sk-test".to_string()),
            timeout_ms: 500,
            ..EvalConfig::default()
        };
        let input = EvaluationInput {
            event_id: "evt-timeout".to_string(),
            event_type: "tool.call".to_string(),
            payload: json!({}),
        };

        let error = evaluate(&input, &config).expect_err("timeout response should fail");
        assert!(
            error.contains("error_type=network_timeout") || error.contains("timed out"),
            "unexpected error message: {error}"
        );
    }

    fn spawn_mock_http_server(
        status: u16,
        body: String,
        response_delay_ms: u64,
    ) -> (String, thread::JoinHandle<()>) {
        let listener =
            TcpListener::bind("127.0.0.1:0").expect("mock http listener should bind successfully");
        let address = listener.local_addr().expect("listener address should be available");

        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0_u8; 4096];
                let _ = stream.read(&mut buffer);
                if response_delay_ms > 0 {
                    thread::sleep(Duration::from_millis(response_delay_ms));
                }
                let response = format!(
                    "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        (format!("http://{}", address), handle)
    }
}
