use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "defaultUserName", default)]
    pub default_user_name: String,

    #[serde(rename = "defaultLength", default = "default_length")]
    pub default_length: u32,

    #[serde(rename = "defaultSymbols", default)]
    pub default_symbols: bool,

    #[serde(rename = "defaultTimeout", default = "default_timeout")]
    pub default_timeout: u32,

    #[serde(rename = "defaultPrint", default)]
    pub default_print: bool,

    #[serde(rename = "defaultRemember", default)]
    pub default_remember: bool,

    #[serde(rename = "sites", default)]
    pub sites: HashMap<String, SiteConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_user_name: String::new(),
            default_length: 16,
            default_symbols: false,
            default_timeout: 45,
            default_print: false,
            default_remember: false,
            sites: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SiteConfig {
    #[serde(rename = "userName", skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    #[serde(rename = "length", skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,

    #[serde(rename = "symbols", skip_serializing_if = "Option::is_none")]
    pub symbols: Option<bool>,
}

pub fn config_path() -> Result<PathBuf, String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "failed to get user config directory".to_string())?;

    Ok(config_dir.join("keyforge").join("config.json"))
}

pub fn get_config() -> Result<Config, String> {
    let path = config_path()?;

    ensure_config_dir(&path)?;

    if !path.exists() {
        let cfg = Config::default();

        let json = serde_json::to_string_pretty(&cfg)
            .map_err(|e| format!("failed to serialize default config: {e}"))?;

        write_config_file(&path, &json)?;
        return Ok(cfg);
    }

    check_config_file_permissions(&path)?;

    let data = fs::read_to_string(&path).map_err(|e| format!("failed to read config: {e}"))?;

    let cfg = serde_json::from_str(&data).map_err(|e| format!("failed to parse config: {e}"))?;

    Ok(cfg)
}

pub fn set_config(cfg: &Config) -> Result<(), String> {
    let path = config_path()?;

    ensure_config_dir(&path)?;

    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("failed to serialize config: {e}"))?;

    write_config_file(&path, &json)?;
    Ok(())
}

pub fn render_config_summary(cfg: &Config, path: &Path) -> String {
    let mut out = String::new();

    writeln!(out, "Config file: {}", path.display()).unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Defaults:").unwrap();
    writeln!(
        out,
        "  {}",
        row("username", &fmt_str_or_none(&cfg.default_user_name))
    )
    .unwrap();
    writeln!(out, "  {}", row("length", &cfg.default_length.to_string())).unwrap();
    writeln!(
        out,
        "  {}",
        row("symbols", &cfg.default_symbols.to_string())
    )
    .unwrap();
    writeln!(
        out,
        "  {}",
        row("timeout", &cfg.default_timeout.to_string())
    )
    .unwrap();
    writeln!(out, "  {}", row("print", &cfg.default_print.to_string())).unwrap();
    writeln!(
        out,
        "  {}",
        row("remember", &cfg.default_remember.to_string())
    )
    .unwrap();
    writeln!(out).unwrap();

    writeln!(out, "Remembered sites:").unwrap();
    if cfg.sites.is_empty() {
        writeln!(out, "  (none)").unwrap();
    } else {
        let mut entries: Vec<_> = cfg.sites.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());

        for (site, sc) in entries {
            writeln!(out, "  {site}:").unwrap();
            writeln!(out, "    {}", row("username", &fmt_opt_str(&sc.user_name))).unwrap();
            writeln!(out, "    {}", row("length", &fmt_opt(&sc.length))).unwrap();
            writeln!(out, "    {}", row("symbols", &fmt_opt(&sc.symbols))).unwrap();
        }
    }

    out
}

fn ensure_config_dir(path: &std::path::Path) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| "failed to get config parent directory".to_string())?;

    fs::create_dir_all(dir).map_err(|e| format!("failed to create config directory: {e}"))?;

    #[cfg(unix)]
    {
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("failed to set config directory permissions: {e}"))?;
    }
    Ok(())
}

fn write_config_file(path: &std::path::Path, json: &str) -> Result<(), String> {
    let mut options = fs::OpenOptions::new();

    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(path)
        .map_err(|e| format!("failed to open config for writing: {e}"))?;

    use std::io::Write;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("failed to write config: {e}"))?;

    #[cfg(unix)]
    {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("failed to set config file permissions: {e}"))?;
    }
    Ok(())
}

fn check_config_file_permissions(path: &std::path::Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let mode = fs::metadata(path)
            .map_err(|e| format!("failed to inspect config permissions: {e}"))?
            .permissions()
            .mode()
            & 0o777;

        if mode != 0o600 {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("failed to fix config file permissions: {e}"))?;
        }
    }
    Ok(())
}

fn default_length() -> u32 {
    16
}
fn default_timeout() -> u32 {
    45
}

fn row(label: &str, value: &str) -> String {
    format!("{label:<8}: {value}")
}

fn fmt_str_or_none(s: &str) -> String {
    if s.is_empty() { "(none)" } else { s }.to_string()
}

fn fmt_opt_str(o: &Option<String>) -> String {
    o.as_deref()
        .map(fmt_str_or_none)
        .unwrap_or_else(|| "(none)".to_string())
}

fn fmt_opt<T: std::fmt::Display>(o: &Option<T>) -> String {
    o.as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "(default)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config_has_expected_values() {
        let cfg = Config::default();

        assert_eq!(cfg.default_user_name, "");
        assert_eq!(cfg.default_length, 16);
        assert!(!cfg.default_symbols);
        assert_eq!(cfg.default_timeout, 45);
        assert!(!cfg.default_print);
        assert!(!cfg.default_remember);
        assert!(cfg.sites.is_empty());
    }

    #[test]
    fn site_config_serializes_expected_fields() {
        let site = SiteConfig {
            user_name: Some("alice".to_string()),
            length: Some(20),
            symbols: Some(true),
        };

        let json = serde_json::to_string(&site).unwrap();
        assert!(json.contains("\"userName\":\"alice\""));
        assert!(json.contains("\"length\":20"));
        assert!(json.contains("\"symbols\":true"));
    }

    #[test]
    fn config_deserializes_missing_fields_with_expected_defaults() {
        let json = r#"{"sites":{}}"#;

        let cfg: Config = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.default_user_name, "");
        assert_eq!(cfg.default_length, 16);
        assert!(!cfg.default_symbols);
        assert_eq!(cfg.default_timeout, 45);
        assert!(!cfg.default_print);
        assert!(!cfg.default_remember);
        assert!(cfg.sites.is_empty());
    }
}
