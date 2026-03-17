use req::parser::{tokenize, process};
use std::io::Write;
use tempfile::NamedTempFile;

fn write_temp_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("failed to write temp file");
    file
}

#[test]
fn tokenize_get_request() {
    let file = write_temp_file("GET http://example.com/api\n");
    let tokens = tokenize(file.path().to_str().unwrap());

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].value.as_deref(), Some("GET"));
    assert_eq!(tokens[1].value.as_deref(), Some("http://example.com/api"));
}

#[test]
fn tokenize_post_with_headers_and_body() {
    let content = "POST http://example.com/api\nContent-Type: application/json\n\n{\"key\": \"value\"}\n";
    let file = write_temp_file(content);
    let tokens = tokenize(file.path().to_str().unwrap());

    assert_eq!(tokens[0].value.as_deref(), Some("POST"));
    assert_eq!(tokens[1].value.as_deref(), Some("http://example.com/api"));
    assert_eq!(tokens[2].value.as_deref(), Some("Content-Type:"));
    assert_eq!(tokens[3].value.as_deref(), Some("application/json"));
}

#[test]
fn tokenize_empty_file() {
    let file = write_temp_file("");
    let tokens = tokenize(file.path().to_str().unwrap());
    assert!(tokens.is_empty());
}

#[test]
fn tokenize_json_body_tokens() {
    let content = "POST http://example.com\nContent-Type: application/json\n\n{\"name\": \"test\"}\n";
    let file = write_temp_file(content);
    let tokens = tokenize(file.path().to_str().unwrap());

    // Body token should be present after the blank line
    assert!(tokens.len() >= 5);
    let body_val = tokens[4].value.as_deref().unwrap();
    assert!(body_val.contains("name") || body_val.contains("{"));
}

#[tokio::test]
async fn process_missing_method() {
    let tokens = vec![];
    let client = reqwest::Client::new();
    let result = process(client, &tokens).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("missing method"));
}

#[tokio::test]
async fn process_missing_url() {
    let tokens = vec![req::parser::Token {
        value: Some("GET".to_string()),
    }];
    let client = reqwest::Client::new();
    let result = process(client, &tokens).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("missing URL"));
}

#[tokio::test]
async fn process_unsupported_method() {
    let tokens = vec![
        req::parser::Token {
            value: Some("HEAD".to_string()),
        },
        req::parser::Token {
            value: Some("http://example.com".to_string()),
        },
    ];
    let client = reqwest::Client::new();
    let result = process(client, &tokens).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unsupported method"));
}
