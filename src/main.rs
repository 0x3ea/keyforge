use keyforge::{
    cli::{self, Commands},
    clipboard, completions,
    config::{self, SiteConfig},
    crypto::{build_salt, generate_key},
    encode::encode,
    term,
};

use clap::{CommandFactory, Parser};

struct ResolvedOptions {
    site: String,
    username: String,
    length: u32,
    symbols: bool,
    timeout: u32,
    print: bool,
    remember: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = cli::Cli::parse();

    if let Some(Commands::Completion { shell }) = args.command {
        let mut cmd = cli::Cli::command();
        let output = completions::generate_completion(shell, &mut cmd)?;
        print!("{output}");
        return Ok(());
    }

    let mut cfg = config::get_config()?;

    let raw_site = args
        .site
        .as_deref()
        .ok_or_else(|| "site is required".to_string())?;

    let site = cli::normalize_site(raw_site)?;
    let site_cfg = cfg.sites.get(&site);

    let username = args
        .username
        .or_else(|| site_cfg.and_then(|s| s.user_name.clone()))
        .unwrap_or_else(|| cfg.default_user_name.clone())
        .to_lowercase();

    let length = args
        .length
        .or_else(|| site_cfg.and_then(|s| s.length))
        .unwrap_or(cfg.default_length);

    let symbols = if args.symbols {
        true
    } else {
        site_cfg
            .and_then(|s| s.symbols)
            .unwrap_or(cfg.default_symbols)
    };

    let options = ResolvedOptions {
        site,
        username,
        length,
        symbols,
        timeout: args.timeout.unwrap_or(cfg.default_timeout),
        print: args.print || cfg.default_print,
        remember: args.remember || cfg.default_remember,
    };
    valid_options(&options)?;

    let password = term::get_master_password(site_cfg.is_none())?;
    let salt = build_salt(&options.site, &options.username);
    let key = generate_key(&password, &salt)?;
    let generated = encode(&key, options.length, options.symbols)?;

    if options.remember {
        cfg.sites.insert(
            options.site.clone(),
            SiteConfig {
                user_name: Some(options.username.clone()),
                length: Some(options.length),
                symbols: Some(options.symbols),
            },
        );
        config::set_config(&cfg)?;
    }

    if options.print {
        let password_text = std::str::from_utf8(generated.as_bytes())
            .map_err(|e| format!("generated password is not valid UTF-8: {e}"))?;

        println!("{password_text}");
    } else {
        clipboard::write_to_clipboard(&generated, options.timeout)?;
    }

    Ok(())
}

fn valid_options(options: &ResolvedOptions) -> Result<(), String> {
    if options.length < 12 {
        return Err("password length must be at least 12".to_string());
    }

    if options.length > 128 {
        return Err("password length must be at most 128".to_string());
    }

    if options.timeout > 3600 {
        return Err("clipboard timeout must be at most 3600".to_string());
    }

    Ok(())
}
