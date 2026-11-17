//! Exercises public configuration parsing and descriptors without callbacks.

use qcontrol::prelude::*;

qcontrol::export_plugin!(PluginBuilder::new("configless-tests"));

/// Plugin-owned schema used to check ownership and Serde's normal rules.
#[derive(Debug, PartialEq, serde::Deserialize)]
struct Settings {
    mode: String,
}

/// A schema whose raw Serde errors can contain rejected input values.
#[derive(Debug, serde::Deserialize)]
enum Mode {
    Active,
}

/// Missing callbacks remain absent in the exported descriptor, not wrappers.
#[test]
fn configless_descriptor_has_null_lifecycle_callbacks() {
    assert!(qcontrol_plugin.0.on_init.is_none());
    assert!(qcontrol_plugin.0.on_config.is_none());
}

/// Decoded settings survive the callback input and preserve normal Serde rules.
#[test]
fn parser_returns_owned_settings_without_imposing_schema_policy() {
    let settings = {
        let json = String::from(r#"{"mode":"active","extra":true}"#);
        parse_config::<Settings>(&json).unwrap()
    };
    assert_eq!(settings.mode, "active");
    assert_eq!(parse_config::<Option<Settings>>("null").unwrap(), None);

    // No defaults or null-to-object conversion are supplied by the SDK.
    for json in ["null", "{}", r#"{"mode":42}"#, "[]"] {
        assert!(parse_config::<Settings>(json).is_err(), "{json}");
    }
}

/// Syntax, type and schema errors never echo input or raw parser diagnostics.
#[test]
fn parser_errors_are_invalid_config_and_keep_rejected_values_private() {
    let secret = "private-config-sentinel";
    let invalid_variant = format!("\"{secret}\"");
    let malformed = format!("{{\"mode\":\"{secret}\"");
    let wrong_type = format!("{{\"mode\":{{\"{secret}\":true}}}}");
    let errors = [
        parse_config::<Mode>(&invalid_variant).unwrap_err(),
        parse_config::<Settings>(&malformed).unwrap_err(),
        parse_config::<Settings>(&wrong_type).unwrap_err(),
    ];
    for error in errors {
        assert!(!error.to_string().contains(secret));
        assert!(std::error::Error::source(&error).is_none());
    }
}

/// Unknown and duplicate fields are named, because field names are not values;
/// the value beside them and the input position stay out of the reason.
#[test]
fn parser_errors_name_unknown_and_duplicate_fields_only() {
    #[derive(Debug, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Strict {
        #[allow(dead_code)]
        mode: Option<String>,
    }
    let secret = "private-config-sentinel";
    let unknown = parse_config::<Strict>(&format!(r#"{{"mood":"{secret}"}}"#)).unwrap_err();
    assert_eq!(unknown.0, "unknown field `mood`, expected `mode`");
    let duplicate =
        parse_config::<Strict>(&format!(r#"{{"mode":"a","mode":"{secret}"}}"#)).unwrap_err();
    assert_eq!(duplicate.0, "duplicate field `mode`");
}
