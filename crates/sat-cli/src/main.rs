use clap::{Parser, Subcommand};
use satcrawler::{Crawler, CrawlerConfig, CrawlerOptions, CrawlerType, Credentials, LoginType};
use std::env;
use std::io::{self, Write};

fn prompt(label: &str) -> String {
    print!("{}", label);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn resolve_path(p: String) -> String {
    let expanded = if p.starts_with("~/") || p == "~" {
        let home = env::var("HOME").unwrap_or_default();
        format!("{}/{}", home, p.trim_start_matches("~/"))
    } else {
        p
    };
    let path = std::path::Path::new(&expanded);
    if path.is_absolute() {
        return expanded;
    }
    env::current_dir()
        .map(|d| d.join(path).to_string_lossy().into_owned())
        .unwrap_or(expanded)
}

#[derive(Parser)]
#[command(name = "sat-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Crawl {
        #[command(subcommand)]
        subcommand: CrawlCommands,
    },
    Config,
    Doctor,
}

#[derive(Subcommand)]
enum CrawlCommands {
    ValidateCredentials {
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
    DownloadInvoices {
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Doctor => {
            println!("sat-cli doctor\n");

            match env::var("TWOCAPTCHA_API_KEY") {
                Ok(_) => println!("  [ok] TWOCAPTCHA_API_KEY is set"),
                Err(_) => {
                    println!("  [!]  TWOCAPTCHA_API_KEY is not set — captcha solving will fail")
                }
            }

            let config = CrawlerConfig::new_from_file();
            if !config.credentials.username.is_empty() && !config.credentials.password.is_empty() {
                println!(
                    "  [ok] Credentials are configured (username: {})",
                    config.credentials.username
                );
            } else if !config.credentials.username.is_empty() {
                println!("  [!]  Username is set but password is missing");
            } else {
                println!("  [!]  No credentials found in ~/sat-cli/config.json");
            }

            if let LoginType::Fiel = config.credentials.login_type {
                match config.credentials.crt_path.as_deref() {
                    Some(p) if std::path::Path::new(p).exists() => {
                        println!("  [ok] Certificate file exists: {}", p)
                    }
                    Some(p) => println!("  [!]  Certificate file not found: {}", p),
                    None => println!("  [!]  FIEL login requires a certificate path (crt_path)"),
                }
                match config.credentials.key_path.as_deref() {
                    Some(p) if std::path::Path::new(p).exists() => {
                        println!("  [ok] Key file exists: {}", p)
                    }
                    Some(p) => println!("  [!]  Key file not found: {}", p),
                    None => println!("  [!]  FIEL login requires a key path (key_path)"),
                }
            }

            match env::var("SATCLI_DOCUMENTS_FOLDER") {
                Ok(path) => println!("  [ok] Custom documents folder: {}", path),
                Err(_) => {
                    println!("  [-]  No custom documents folder set — using ~/sat-cli/documents/")
                }
            }
        }
        Commands::Config => {
            println!("sat-cli config\n");

            let login_type_input = prompt("Login type [ciec/fiel] (default: ciec): ");
            let login_type = if login_type_input.eq_ignore_ascii_case("fiel") {
                LoginType::Fiel
            } else {
                LoginType::Ciec
            };

            let username = prompt("RFC (username): ");
            let password = rpassword::prompt_password("Password: ").unwrap();

            let (crt_path, key_path) = match login_type {
                LoginType::Fiel => {
                    let crt = resolve_path(prompt("Certificate path (.cer): "));
                    let key = resolve_path(prompt("Private key path (.key): "));
                    (Some(crt), Some(key))
                }
                LoginType::Ciec => (None, None),
            };

            CrawlerConfig::new(
                Credentials {
                    login_type,
                    username: username.clone(),
                    password,
                    crt_path,
                    key_path,
                },
                CrawlerOptions {
                    headless: true,
                    sandbox: true,
                },
            );
            println!("\nConfig saved to ~/sat-cli/config.json");
            println!("  username: {}", username);
        }
        Commands::Crawl { subcommand } => match subcommand {
            CrawlCommands::ValidateCredentials { username, password } => {
                let mut config = CrawlerConfig::new_from_file();
                if let Some(custom_username) = username {
                    config.credentials.username = custom_username;
                }
                if let Some(custom_password) = password {
                    config.credentials.password = custom_password;
                }
                let crawler = Crawler::new(CrawlerType::ValidateCredentials, config);
                let response = crawler.run().await;
                println!(
                    "{}",
                    serde_json::to_string(&response).expect("Response serialization error")
                );
            }
            CrawlCommands::DownloadInvoices { username, password } => {
                let mut config = CrawlerConfig::new_from_file();
                if let Some(custom_username) = username {
                    config.credentials.username = custom_username;
                }
                if let Some(custom_password) = password {
                    config.credentials.password = custom_password;
                }
                let crawler = Crawler::new(CrawlerType::DownloadInvoices, config);
                let response = crawler.run().await;
                println!(
                    "{}",
                    serde_json::to_string(&response).expect("Response serialization error")
                );
            }
        },
    }
}
