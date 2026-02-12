// Configuration loading with defined precedence hierarchy:
//
//   CLI flags  >  Environment variables  >  Config file  >  Defaults
//
// Supported config file formats:
// - TOML (nested): detected by .toml extension
// - JSON (flat):   detected by .json extension
//
// Environment variable override pattern: APP__SECTION__KEY
// Example: APP__DATABASE__URL=sqlite:///data/budget.db

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default = "default_database")]
    pub database: DatabaseConfig,
    #[serde(default = "default_currency")]
    pub currency: CurrencyConfig,
    #[serde(default = "default_cors")]
    pub cors: CorsConfig,
    #[serde(default = "default_ui")]
    pub ui: UiConfig,
}

fn default_server() -> ServerConfig {
    ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 3000,
    }
}

fn default_database() -> DatabaseConfig {
    DatabaseConfig {
        url: "sqlite://data/budget.db".to_string(),
    }
}

fn default_currency() -> CurrencyConfig {
    CurrencyConfig {
        code: "PLN".to_string(),
        minor_unit_name: "grosz".to_string(),
        decimal_places: 2,
    }
}

fn default_cors() -> CorsConfig {
    CorsConfig {
        allowed_origins: vec![],
    }
}

fn default_ui() -> UiConfig {
    UiConfig {
        budget_bars: default_budget_bars(),
    }
}

fn default_budget_bars() -> BudgetBarsConfig {
    BudgetBarsConfig {
        green_threshold: 80,
        yellow_threshold: 100,
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_url")]
    pub url: String,
}

fn default_db_url() -> String {
    "sqlite://data/budget.db".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct CurrencyConfig {
    #[serde(default = "default_currency_code")]
    pub code: String,
    #[serde(default = "default_minor_unit_name")]
    pub minor_unit_name: String,
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u8,
}

fn default_currency_code() -> String {
    "PLN".to_string()
}

fn default_minor_unit_name() -> String {
    "grosz".to_string()
}

fn default_decimal_places() -> u8 {
    2
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UiConfig {
    #[serde(default = "default_budget_bars")]
    pub budget_bars: BudgetBarsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BudgetBarsConfig {
    #[serde(default = "default_green_threshold")]
    pub green_threshold: u8,
    #[serde(default = "default_yellow_threshold")]
    pub yellow_threshold: u8,
}

fn default_green_threshold() -> u8 {
    80
}

fn default_yellow_threshold() -> u8 {
    100
}

/// Convert flat JSON keys (e.g., "server_host") into nested structure
/// (e.g., {"server": {"host": ...}}) so they deserialize into `AppConfig`.
///
/// Mapping rules:
///   server_host             -> server.host
///   server_port             -> server.port
///   database_url            -> database.url
///   currency_code           -> currency.code
///   currency_minor_unit_name -> currency.minor_unit_name (split on first `_` only)
///   currency_decimal_places -> currency.decimal_places
fn convert_flat_json(value: serde_json::Value) -> serde_json::Value {
    let obj = match value {
        serde_json::Value::Object(map) => map,
        other => return other,
    };

    // Known section prefixes for unambiguous splitting
    let known_sections = ["server", "database", "currency", "cors", "ui"];

    let mut nested: HashMap<String, HashMap<String, serde_json::Value>> = HashMap::new();

    for (key, val) in obj {
        let mut matched = false;
        for section in &known_sections {
            let prefix = format!("{section}_");
            if let Some(field) = key.strip_prefix(&prefix) {
                nested
                    .entry(section.to_string())
                    .or_default()
                    .insert(field.to_string(), val.clone());
                matched = true;
                break;
            }
        }
        if !matched {
            tracing::warn!("Unknown flat config key ignored: {key}");
        }
    }

    let mut result = serde_json::Map::new();
    for (section, fields) in nested {
        let section_obj: serde_json::Map<String, serde_json::Value> = fields.into_iter().collect();
        result.insert(section, serde_json::Value::Object(section_obj));
    }

    serde_json::Value::Object(result)
}

impl AppConfig {
    /// Load configuration with the following precedence (highest wins):
    /// CLI flags > Environment variables > Config file > Defaults
    pub fn load(
        config_path: Option<&str>,
        explicit: bool,
    ) -> Result<Self, config::ConfigError> {
        let mut builder = config::Config::builder();

        if let Some(path) = config_path {
            let path_ref = Path::new(path);
            let exists = path_ref.exists();

            if !exists && explicit {
                return Err(config::ConfigError::NotFound(format!(
                    "Config file not found: {path}"
                )));
            }

            if exists {
                let ext = path_ref
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                match ext {
                    "json" => {
                        let contents = std::fs::read_to_string(path).map_err(|e| {
                            config::ConfigError::Message(format!("Failed to read {path}: {e}"))
                        })?;
                        let raw: serde_json::Value =
                            serde_json::from_str(&contents).map_err(|e| {
                                config::ConfigError::Message(format!(
                                    "Invalid JSON in {path}: {e}"
                                ))
                            })?;
                        let nested = convert_flat_json(raw);
                        let nested_str = serde_json::to_string(&nested).map_err(|e| {
                            config::ConfigError::Message(format!(
                                "Failed to serialize nested config: {e}"
                            ))
                        })?;
                        builder = builder.add_source(config::File::from_str(
                            &nested_str,
                            config::FileFormat::Json,
                        ));
                    }
                    _ => {
                        builder = builder.add_source(
                            config::File::with_name(path)
                                .format(config::FileFormat::Toml)
                                .required(true),
                        );
                    }
                }
            }
        }

        // Environment variables always override config file
        builder = builder.add_source(
            config::Environment::with_prefix("APP").separator("__"),
        );

        let config = builder.build()?;
        let app_config: Self = config.try_deserialize()?;
        app_config.validate()?;
        Ok(app_config)
    }

    fn validate(&self) -> Result<(), config::ConfigError> {
        let bars = &self.ui.budget_bars;
        if bars.green_threshold == 0 {
            return Err(config::ConfigError::Message(
                "ui.budget_bars.green_threshold must be greater than 0".to_string(),
            ));
        }
        if bars.yellow_threshold == 0 {
            return Err(config::ConfigError::Message(
                "ui.budget_bars.yellow_threshold must be greater than 0".to_string(),
            ));
        }
        if bars.green_threshold >= bars.yellow_threshold {
            return Err(config::ConfigError::Message(format!(
                "ui.budget_bars.green_threshold ({}) must be less than yellow_threshold ({})",
                bars.green_threshold, bars.yellow_threshold
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_toml(content: &str) -> NamedTempFile {
        let mut f = tempfile::Builder::new()
            .suffix(".toml")
            .tempfile()
            .unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn default_budget_bar_thresholds() {
        let config = AppConfig::load(None, false).unwrap();
        assert_eq!(config.ui.budget_bars.green_threshold, 80);
        assert_eq!(config.ui.budget_bars.yellow_threshold, 100);
    }

    #[test]
    fn custom_budget_bar_thresholds_from_toml() {
        let f = write_toml(
            r#"
[ui.budget_bars]
green_threshold = 70
yellow_threshold = 90
"#,
        );
        let config = AppConfig::load(Some(f.path().to_str().unwrap()), true).unwrap();
        assert_eq!(config.ui.budget_bars.green_threshold, 70);
        assert_eq!(config.ui.budget_bars.yellow_threshold, 90);
    }

    #[test]
    fn rejects_green_gte_yellow() {
        let f = write_toml(
            r#"
[ui.budget_bars]
green_threshold = 100
yellow_threshold = 80
"#,
        );
        let err = AppConfig::load(Some(f.path().to_str().unwrap()), true).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("green_threshold"), "unexpected error: {msg}");
        assert!(msg.contains("less than"), "unexpected error: {msg}");
    }

    #[test]
    fn rejects_equal_thresholds() {
        let f = write_toml(
            r#"
[ui.budget_bars]
green_threshold = 80
yellow_threshold = 80
"#,
        );
        let err = AppConfig::load(Some(f.path().to_str().unwrap()), true).unwrap_err();
        assert!(err.to_string().contains("less than"));
    }

    #[test]
    fn rejects_zero_green_threshold() {
        let f = write_toml(
            r#"
[ui.budget_bars]
green_threshold = 0
yellow_threshold = 100
"#,
        );
        let err = AppConfig::load(Some(f.path().to_str().unwrap()), true).unwrap_err();
        assert!(err.to_string().contains("greater than 0"));
    }

    #[test]
    fn partial_budget_bars_config_uses_defaults() {
        // Only set green_threshold; yellow_threshold should use default
        let f = write_toml(
            r#"
[ui.budget_bars]
green_threshold = 50
"#,
        );
        let config = AppConfig::load(Some(f.path().to_str().unwrap()), true).unwrap();
        assert_eq!(config.ui.budget_bars.green_threshold, 50);
        assert_eq!(config.ui.budget_bars.yellow_threshold, 100);
    }
}
