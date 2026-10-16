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
use std::path::{Path, PathBuf};

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
    #[serde(default = "default_transaction")]
    pub transaction: TransactionConfig,
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

fn default_transaction() -> TransactionConfig {
    TransactionConfig {
        import: default_import(),
    }
}

fn default_import() -> ImportConfig {
    ImportConfig {
        rules_file: default_rules_file(),
    }
}

/// Default `rules_file` path: `rules.toml` next to the config file, used
/// when `[transaction.import]` doesn't set `rules_file` explicitly.
fn default_rules_file() -> PathBuf {
    PathBuf::from("rules.toml")
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

/// The `[transaction]` config section: currently only CSV-import-related
/// settings, nested under `[transaction.import]`.
#[derive(Debug, Deserialize, Clone)]
pub struct TransactionConfig {
    #[serde(default = "default_import")]
    pub import: ImportConfig,
}

/// CSV import categorization settings (`[transaction.import]`).
///
/// `rules_file` names a hand-editable TOML file (see
/// `csv_import::load_rules_or_empty`/`save_rules`) holding the
/// categorization rules, rather than embedding them inline — this lets the
/// bulk-categorize API (section 3) persist newly learned rules without
/// rewriting the whole app config. After [`AppConfig::load`] resolves it,
/// this field is always an absolute path (see [`resolve_relative`]).
#[derive(Debug, Deserialize, Clone)]
pub struct ImportConfig {
    /// Path to the rules file, relative to the config file's own directory
    /// (absolute paths are used as-is). Defaults to `rules.toml` next to
    /// the config file.
    #[serde(default = "default_rules_file")]
    pub rules_file: PathBuf,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            rules_file: default_rules_file(),
        }
    }
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

/// Top-level `include = [...]` directive read from a TOML config file, e.g.
/// `include = ["./rules.toml", "/etc/otter/shared.toml"]`.
#[derive(Debug, Deserialize, Default)]
struct IncludeDirective {
    #[serde(default)]
    include: Vec<String>,
}

/// Resolves the `include` paths listed at the top of `config_file`, relative
/// to `config_file`'s own parent directory (absolute paths are used as-is).
///
/// Only one level of includes is supported — an included file's own
/// `include` list, if any, is ignored. Good enough for splitting out things
/// like categorization rules; add recursive resolution if nested includes
/// are ever needed.
fn resolve_includes(config_file: &Path) -> Result<Vec<PathBuf>, config::ConfigError> {
    let directive_source = config::Config::builder()
        .add_source(
            config::File::with_name(config_file.to_string_lossy().as_ref())
                .format(config::FileFormat::Toml)
                .required(true),
        )
        .build()?;
    let directive: IncludeDirective = directive_source.try_deserialize()?;

    tracing::debug!(
        "Found {} include paths in {}",
        directive.include.len(),
        config_file.display()
    );

    let base_dir = config_file.parent().unwrap_or_else(|| Path::new("."));
    Ok(directive
        .include
        .into_iter()
        .map(|p| resolve_relative(base_dir, &PathBuf::from(p)))
        .collect())
}

/// Resolves `path` against `base_dir`: an absolute `path` is used as-is,
/// a relative one is joined onto `base_dir`. Shared by `include` path
/// resolution and [`ImportConfig::rules_file`] resolution, both of which
/// need "relative to the config file's own directory" semantics.
fn resolve_relative(base_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

impl AppConfig {
    /// Load configuration with the following precedence (highest wins):
    /// CLI flags > Environment variables > Config file > Defaults
    pub fn load(config_path: Option<&str>, explicit: bool) -> Result<Self, config::ConfigError> {
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
                let ext = path_ref.extension().and_then(|e| e.to_str()).unwrap_or("");

                match ext {
                    "json" => {
                        let contents = std::fs::read_to_string(path).map_err(|e| {
                            config::ConfigError::Message(format!("Failed to read {path}: {e}"))
                        })?;
                        let raw: serde_json::Value =
                            serde_json::from_str(&contents).map_err(|e| {
                                config::ConfigError::Message(format!("Invalid JSON in {path}: {e}"))
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
                    // ponytail: JSON config has no `include` support — it's the
                    // flat/legacy format, not used for multi-file setups.
                    _ => {
                        for include_path in resolve_includes(path_ref)? {
                            builder =
                                builder.add_source(config::File::from(include_path).required(true));
                        }
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
        builder = builder.add_source(config::Environment::with_prefix("APP").separator("__"));

        let config = builder.build()?;
        let mut app_config: Self = config.try_deserialize()?;

        let base_dir = config_path
            .map(|p| {
                Path::new(p)
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            })
            .unwrap_or_else(|| PathBuf::from("."));
        app_config.transaction.import.rules_file =
            resolve_relative(&base_dir, &app_config.transaction.import.rules_file);

        app_config.validate()?;
        tracing::info!("Loaded configuration: {:#?}", app_config);
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
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
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

    /// With no `rules_file` set, the default `rules.toml` resolves to a
    /// path next to the config file (not relative to the process's CWD).
    #[test]
    fn default_rules_file_resolves_relative_to_config_dir() {
        let dir = tempfile::tempdir().unwrap();
        let main_path = dir.path().join("config.toml");
        std::fs::write(&main_path, "").unwrap();

        let config = AppConfig::load(Some(main_path.to_str().unwrap()), true).unwrap();
        assert_eq!(
            config.transaction.import.rules_file,
            dir.path().join("rules.toml")
        );
    }

    /// A relative `rules_file` is resolved against the config file's own
    /// directory, not the process's CWD.
    #[test]
    fn relative_rules_file_resolves_against_config_dir() {
        let dir = tempfile::tempdir().unwrap();
        let main_path = dir.path().join("config.toml");
        std::fs::write(
            &main_path,
            r#"
[transaction.import]
rules_file = "custom-rules.toml"
"#,
        )
        .unwrap();

        let config = AppConfig::load(Some(main_path.to_str().unwrap()), true).unwrap();
        assert_eq!(
            config.transaction.import.rules_file,
            dir.path().join("custom-rules.toml")
        );
    }

    /// An absolute `rules_file` is used as-is, regardless of the config
    /// file's directory.
    #[test]
    fn absolute_rules_file_is_used_as_is() {
        let dir = tempfile::tempdir().unwrap();
        let main_path = dir.path().join("config.toml");
        let absolute_rules = dir.path().join("elsewhere").join("rules.toml");
        std::fs::write(
            &main_path,
            format!(
                "[transaction.import]\nrules_file = {:?}\n",
                absolute_rules.to_str().unwrap()
            ),
        )
        .unwrap();

        let config = AppConfig::load(Some(main_path.to_str().unwrap()), true).unwrap();
        assert_eq!(config.transaction.import.rules_file, absolute_rules);
    }

    /// With no config file at all, the default `rules_file` resolves
    /// relative to the process's CWD (`"."`).
    #[test]
    fn default_rules_file_with_no_config_resolves_to_cwd() {
        let config = AppConfig::load(None, false).unwrap();
        assert_eq!(
            config.transaction.import.rules_file,
            PathBuf::from(".").join("rules.toml")
        );
    }
}
