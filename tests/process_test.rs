use wiremock::matchers::{body_string, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use req::parser::{process, Token, TokenType};

#[tokio::test]
async fn unsupported_method() {
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
    let result = process(client, &tokens).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unsupported method"));
}

#[tokio::test]
async fn empty_tokens() {
    let tokens = vec![];
    let client = reqwest::Client::new();
    let result = process(client, &tokens).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unsupported method"));
}

#[tokio::test]
async fn get_request() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_string("world"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "GET".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/hello", server.uri()),
        },
    ];

    let client = reqwest::Client::new();
    let result = process(client, &tokens).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "world");
}

#[tokio::test]
async fn post_request() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/create"))
        .and(body_string(r#"{"item": "new"}"#))
        .respond_with(ResponseTemplate::new(201).set_body_string("created"))
        .mount(&server)
        .await;

    let tokens = vec![
        Token {
            token_type: TokenType::Method,
            value: "POST".to_string(),
        },
        Token {
            token_type: TokenType::URL,
            value: format!("{}/create", server.uri()),
        },
        Token {
            token_type: TokenType::Body,
            value: r#"{"item": "new"}"#.to_string(),
        },
    ];

    let client = reqwest::Client::new();
    let result = process(client, &tokens).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "created");
}
