use clap::{Parser, Subcommand};
use clap_complete::Shell;
use url::Url;
#[derive(Parser, Debug)]
#[command(name = "keyforge")]
#[command(version)]
#[command(about = "Deterministic password generator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    pub site: Option<String>,

    #[arg(short = 'u', long = "username")]
    pub username: Option<String>,

    #[arg(short = 'l', long = "length")]
    pub length: Option<u32>,

    #[arg(short = 's', long = "symbols")]
    pub symbols: bool,

    #[arg(short = 'p', long = "print")]
    pub print: bool,

    #[arg(short = 'r', long = "remember")]
    pub remember: bool,

    #[arg(long = "timeout")]
    pub timeout: Option<u32>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub fn normalize_site(raw: &str) -> Result<String, String> {
    let raw = raw.trim();

    if raw.is_empty() {
        return Err("site cannot be empty".to_string());
    }

    let with_scheme = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };

    let url = Url::parse(&with_scheme).map_err(|e| e.to_string())?;

    let host = url
        .host_str()
        .ok_or_else(|| "site must contain a hostname".to_string())?;

    Ok(host.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_site_hostname() {
        assert_eq!(normalize_site("Github.com").unwrap(), "github.com");
        assert_eq!(
            normalize_site("https://GitHub.com/path").unwrap(),
            "github.com"
        );
        assert_eq!(normalize_site(" github.com   ").unwrap(), "github.com");
    }
    #[test]
    fn rejects_empty_site() {
        assert!(normalize_site(" ").is_err());
        assert!(normalize_site("    ").is_err());
    }
}
