use std::collections::HashMap;
use std::fs;

use tempfile::TempDir;

use req::parser::{load_env_vars, substitute};

fn setup_env_files(dir: &TempDir, base: &str, private: Option<&str>) -> String {
    fs::write(dir.path().join("http-client.env.json"), base).unwrap();
    if let Some(p) = private {
        fs::write(dir.path().join("http-client.private.env.json"), p).unwrap();
    }
    // Return a fake .rest file path inside the temp dir
    let rest_path = dir.path().join("test.rest");
    fs::write(&rest_path, "").unwrap();
    rest_path.to_str().unwrap().to_string()
}

#[test]
fn loads_development_env() {
    let dir = TempDir::new().unwrap();
    let rest_file = setup_env_files(
        &dir,
        r#"{
            "development": {
                "host": "localhost",
                "id-value": 12345
            }
        }"#,
        None,
    );

    let vars = load_env_vars(&rest_file, "development", false);

    assert_eq!(vars.get("host").unwrap(), "localhost");
    assert_eq!(vars.get("id-value").unwrap(), "12345");
}

#[test]
fn loads_production_env() {
    let dir = TempDir::new().unwrap();
    let rest_file = setup_env_files(
        &dir,
        r#"{
            "development": { "host": "localhost" },
            "production": { "host": "example.com" }
        }"#,
        None,
    );

    let vars = load_env_vars(&rest_file, "production", false);

    assert_eq!(vars.get("host").unwrap(), "example.com");
}

#[test]
fn returns_empty_for_missing_env() {
    let dir = TempDir::new().unwrap();
    let rest_file = setup_env_files(
        &dir,
        r#"{ "development": { "host": "localhost" } }"#,
        None,
    );

    let vars = load_env_vars(&rest_file, "staging", false);

    assert!(vars.is_empty());
}

#[test]
fn returns_empty_when_no_env_file() {
    let dir = TempDir::new().unwrap();
    let rest_path = dir.path().join("test.rest");
    fs::write(&rest_path, "").unwrap();

    let vars = load_env_vars(rest_path.to_str().unwrap(), "development", false);

    assert!(vars.is_empty());
}

#[test]
fn private_overrides_base() {
    let dir = TempDir::new().unwrap();
    let rest_file = setup_env_files(
        &dir,
        r#"{
            "development": {
                "host": "localhost",
                "my-var": "base-value"
            }
        }"#,
        Some(r#"{
            "development": {
                "host": "private-host",
                "password": "secret123"
            }
        }"#),
    );

    let vars = load_env_vars(&rest_file, "development", true);

    assert_eq!(vars.get("host").unwrap(), "private-host"); // overridden
    assert_eq!(vars.get("my-var").unwrap(), "base-value"); // kept from base
    assert_eq!(vars.get("password").unwrap(), "secret123"); // added from private
}

#[test]
fn private_not_loaded_when_flag_is_false() {
    let dir = TempDir::new().unwrap();
    let rest_file = setup_env_files(
        &dir,
        r#"{ "development": { "host": "localhost" } }"#,
        Some(r#"{ "development": { "host": "private-host" } }"#),
    );

    let vars = load_env_vars(&rest_file, "development", false);

    assert_eq!(vars.get("host").unwrap(), "localhost");
}

#[test]
fn substitute_replaces_variables() {
    let mut vars = HashMap::new();
    vars.insert("host".to_string(), "localhost".to_string());
    vars.insert("id-value".to_string(), "12345".to_string());

    let result = substitute("http://{{host}}/api?id={{id-value}}", &vars);

    assert_eq!(result, "http://localhost/api?id=12345");
}

#[test]
fn substitute_leaves_unknown_variables() {
    let vars = HashMap::new();

    let result = substitute("http://{{host}}/api", &vars);

    assert_eq!(result, "http://{{host}}/api");
}

#[test]
fn substitute_handles_spaces_in_braces() {
    let mut vars = HashMap::new();
    vars.insert("host".to_string(), "localhost".to_string());

    let result = substitute("http://{{ host }}/api", &vars);

    assert_eq!(result, "http://localhost/api");
}

#[test]
fn substitute_no_placeholders() {
    let vars = HashMap::new();

    let result = substitute("http://localhost/api", &vars);

    assert_eq!(result, "http://localhost/api");
}
