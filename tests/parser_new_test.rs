use req::parser::{Token, TokenType, tokenize};
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
    assert!(matches!(tokens[0].token_type, TokenType::Method));
    assert_eq!(tokens[0].value, "GET");
    assert!(matches!(tokens[1].token_type, TokenType::URL));
    assert_eq!(tokens[1].value, "http://example.com/api");
}

#[test]
fn tokenize_post_request_with_headers_and_body() {
    let content = "POST http://example.com/api\nUser-Agent: test-agent\n\n{\"key\": \"value\"}\n";
    let file = write_temp_file(content);
    let tokens = tokenize(file.path().to_str().unwrap());

    assert_eq!(tokens.len(), 5);
    assert!(matches!(tokens[0].token_type, TokenType::Method));
    assert_eq!(tokens[0].value, "POST");
    assert!(matches!(tokens[1].token_type, TokenType::URL));
    assert_eq!(tokens[1].value, "http://example.com/api");
    assert!(matches!(tokens[2].token_type, TokenType::Header));
    assert_eq!(tokens[2].value, "User-Agent");
    assert!(matches!(tokens[3].token_type, TokenType::HeaderValue));
    assert_eq!(tokens[3].value, "test-agent");
    assert!(matches!(tokens[4].token_type, TokenType::Body));
    assert!(tokens[4].value.contains("\"key\""));
}

#[test]
fn tokenize_delete_request() {
    let file = write_temp_file("DELETE http://example.com/api/123\n");
    let tokens = tokenize(file.path().to_str().unwrap());

    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].token_type, TokenType::Method));
    assert_eq!(tokens[0].value, "DELETE");
    assert!(matches!(tokens[1].token_type, TokenType::URL));
    assert_eq!(tokens[1].value, "http://example.com/api/123");
}

#[test]
fn tokenize_put_request() {
    let file = write_temp_file("PUT http://example.com/api/1\n");
    let tokens = tokenize(file.path().to_str().unwrap());

    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].token_type, TokenType::Method));
    assert_eq!(tokens[0].value, "PUT");
}

#[test]
fn tokenize_multiple_headers() {
    let content = "GET http://example.com\nHost: example.com\nAccept: text/html\n";
    let file = write_temp_file(content);
    let tokens = tokenize(file.path().to_str().unwrap());

    assert_eq!(tokens.len(), 6);
    assert!(matches!(tokens[2].token_type, TokenType::Header));
    assert_eq!(tokens[2].value, "Host");
    assert!(matches!(tokens[3].token_type, TokenType::HeaderValue));
    assert_eq!(tokens[3].value, "example.com");
    assert!(matches!(tokens[4].token_type, TokenType::Header));
    assert_eq!(tokens[4].value, "Accept");
    assert!(matches!(tokens[5].token_type, TokenType::HeaderValue));
    assert_eq!(tokens[5].value, "text/html");
}

#[test]
fn tokenize_empty_file() {
    let file = write_temp_file("");
    let tokens = tokenize(file.path().to_str().unwrap());
    assert!(tokens.is_empty());
}

#[test]
fn tokenize_body_multiline() {
    let content =
        "POST http://example.com/api\nUser-Agent: bot\n\n{\"name\": \"test\",\n\"age\": 25}\n";
    let file = write_temp_file(content);
    let tokens = tokenize(file.path().to_str().unwrap());

    let body_token = tokens
        .iter()
        .find(|t| matches!(t.token_type, TokenType::Body));
    assert!(body_token.is_some());
    let body = &body_token.unwrap().value;
    assert!(body.contains("\"name\""));
    assert!(body.contains("\"age\""));
}

#[tokio::test]
async fn process_unsupported_method() {
    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "OPTIONS".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: "http://example.com".to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let result = req::parser::process(client, &tokens).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("unsupported method")
    );
}

#[tokio::test]
async fn process_empty_tokens() {
    let tokens = vec![];
    let client = reqwest::Client::new();
    let result = req::parser::process(client, &tokens).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("unsupported method")
    );
}
