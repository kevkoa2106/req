use std::io::Write;

use tempfile::NamedTempFile;

use req::parser::{TokenType, tokenize};

fn write_temp_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("failed to write temp file");
    file
}

// --- Single request tests (updated for Vec<Vec<Token>>) ---

#[test]
fn get_request() {
    let file = write_temp_file("GET http://example.com/api\n");
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 1);
    let tokens = &requests[0];
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].token_type, TokenType::Method));
    assert_eq!(tokens[0].value, "GET");
    assert!(matches!(tokens[1].token_type, TokenType::URL));
    assert_eq!(tokens[1].value, "http://example.com/api");
}

#[test]
fn post_with_headers_and_body() {
    let content = "POST http://example.com/api\nUser-Agent: test-agent\n\n{\"key\": \"value\"}\n";
    let file = write_temp_file(content);
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 1);
    let tokens = &requests[0];
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
fn delete_request() {
    let file = write_temp_file("DELETE http://example.com/api/123\n");
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 1);
    let tokens = &requests[0];
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].token_type, TokenType::Method));
    assert_eq!(tokens[0].value, "DELETE");
    assert!(matches!(tokens[1].token_type, TokenType::URL));
    assert_eq!(tokens[1].value, "http://example.com/api/123");
}

#[test]
fn put_request() {
    let file = write_temp_file("PUT http://example.com/api/1\n");
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 1);
    let tokens = &requests[0];
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].token_type, TokenType::Method));
    assert_eq!(tokens[0].value, "PUT");
}

#[test]
fn multiple_headers() {
    let content = "GET http://example.com\nHost: example.com\nAccept: text/html\n";
    let file = write_temp_file(content);
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 1);
    let tokens = &requests[0];
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
fn empty_file() {
    let file = write_temp_file("");
    let requests = tokenize(file.path().to_str().unwrap());
    assert!(requests.is_empty());
}

#[test]
fn body_multiline() {
    let content =
        "POST http://example.com/api\nUser-Agent: bot\n\n{\"name\": \"test\",\n\"age\": 25}\n";
    let file = write_temp_file(content);
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 1);
    let body_token = requests[0]
        .iter()
        .find(|t| matches!(t.token_type, TokenType::Body));
    assert!(body_token.is_some());
    let body = &body_token.unwrap().value;
    assert!(body.contains("\"name\""));
    assert!(body.contains("\"age\""));
}

// --- Multiple request tests ---

#[test]
fn two_get_requests_separated_by_separator() {
    let content = "GET http://example.com/one\n###\nGET http://example.com/two\n";
    let file = write_temp_file(content);
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0][1].value, "http://example.com/one");
    assert_eq!(requests[1][1].value, "http://example.com/two");
}

#[test]
fn three_requests_different_methods() {
    let content = "GET http://example.com/1\n\
                   ###\n\
                   POST http://example.com/2\n\
                   ###\n\
                   DELETE http://example.com/3\n";
    let file = write_temp_file(content);
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0][0].value, "GET");
    assert_eq!(requests[1][0].value, "POST");
    assert_eq!(requests[2][0].value, "DELETE");
}

#[test]
fn request_with_body_then_separator_then_get() {
    let content = "POST http://example.com/api\n\
                   Content-Type: application/json\n\
                   \n\
                   {\"key\": \"value\"}\n\
                   ###\n\
                   GET http://example.com/other\n";
    let file = write_temp_file(content);
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 2);

    // First request: POST with body
    assert_eq!(requests[0][0].value, "POST");
    let body = requests[0]
        .iter()
        .find(|t| matches!(t.token_type, TokenType::Body));
    assert!(body.is_some());
    assert!(body.unwrap().value.contains("\"key\""));

    // Second request: GET
    assert_eq!(requests[1][0].value, "GET");
    assert_eq!(requests[1][1].value, "http://example.com/other");
}

#[test]
fn separator_with_no_trailing_request() {
    let content = "GET http://example.com\n###\n";
    let file = write_temp_file(content);
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 1);
}

#[test]
fn each_request_has_independent_headers() {
    let content = "GET http://example.com/1\n\
                   Accept: text/html\n\
                   ###\n\
                   POST http://example.com/2\n\
                   Content-Type: application/json\n";
    let file = write_temp_file(content);
    let requests = tokenize(file.path().to_str().unwrap());

    assert_eq!(requests.len(), 2);

    let first_headers: Vec<&str> = requests[0]
        .iter()
        .filter(|t| matches!(t.token_type, TokenType::Header))
        .map(|t| t.value.as_str())
        .collect();
    assert_eq!(first_headers, vec!["Accept"]);

    let second_headers: Vec<&str> = requests[1]
        .iter()
        .filter(|t| matches!(t.token_type, TokenType::Header))
        .map(|t| t.value.as_str())
        .collect();
    assert_eq!(second_headers, vec!["Content-Type"]);
}
