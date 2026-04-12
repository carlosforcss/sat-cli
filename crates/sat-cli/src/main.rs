use clap::{Parser, Subcommand};
use satcrawler::{
    parse_date, Crawler, CrawlerConfig, CrawlerFilters, CrawlerOptions, CrawlerType, Credentials,
    LoginType,
};
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

fn parse_login_type(s: &str) -> Result<LoginType, String> {
    if s.eq_ignore_ascii_case("ciec") {
        Ok(LoginType::Ciec)
    } else if s.eq_ignore_ascii_case("fiel") {
        Ok(LoginType::Fiel)
    } else {
        Err(format!(
            "Invalid login type '{}': expected 'ciec' or 'fiel'",
            s
        ))
    }
}

fn validate_config_or_exit(config: &CrawlerConfig) {
    if let Err(e) = config.validate() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

async fn run_crawl_command(
    crawler_type: CrawlerType,
    args: CrawlArgs,
    filters: Option<CrawlerFilters>,
) {
    let mut config = CrawlerConfig::new_from_file();
    apply_args_to_config(&mut config, args);
    validate_config_or_exit(&config);
    let crawler = Crawler::new(crawler_type, config).with_filters(filters);
    let response = crawler.run().await;
    println!(
        "{}",
        serde_json::to_string(&response).expect("Response serialization error")
    );
}

fn apply_args_to_config(config: &mut CrawlerConfig, args: CrawlArgs) {
    if let Some(v) = args.username {
        config.credentials.username = v;
    }
    if let Some(v) = args.password {
        config.credentials.password = v;
    }
    if let Some(v) = args.crt {
        config.credentials.crt_path = Some(resolve_path(v));
    }
    if let Some(v) = args.key {
        config.credentials.key_path = Some(resolve_path(v));
    }
    if let Some(v) = args.login_type {
        config.credentials.login_type = v;
    }
    if args.headless {
        config.options.headless = true;
    }
    if args.sandbox {
        config.options.sandbox = true;
    }
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

#[derive(Parser)]
struct CrawlArgs {
    #[arg(long)]
    username: Option<String>,
    #[arg(long)]
    password: Option<String>,
    #[arg(long)]
    crt: Option<String>,
    #[arg(long)]
    key: Option<String>,
    #[arg(long, value_parser = parse_login_type)]
    login_type: Option<LoginType>,
    #[arg(long)]
    headless: bool,
    #[arg(long)]
    sandbox: bool,
}

#[derive(Parser)]
struct DownloadFilterArgs {
    #[arg(long, value_parser = parse_date, value_name = "dd/mm/YYYY")]
    start_date: Option<chrono::NaiveDate>,
    #[arg(long, value_parser = parse_date, value_name = "dd/mm/YYYY")]
    end_date: Option<chrono::NaiveDate>,
}

impl From<DownloadFilterArgs> for CrawlerFilters {
    fn from(f: DownloadFilterArgs) -> Self {
        CrawlerFilters {
            start_date: f.start_date,
            end_date: f.end_date,
        }
    }
}

#[derive(Subcommand)]
enum CrawlCommands {
    ValidateCredentials {
        #[command(flatten)]
        args: CrawlArgs,
    },
    DownloadInvoices {
        #[command(flatten)]
        args: CrawlArgs,
        #[command(flatten)]
        filters: DownloadFilterArgs,
    },
    DownloadIssuedInvoices {
        #[command(flatten)]
        args: CrawlArgs,
        #[command(flatten)]
        filters: DownloadFilterArgs,
    },
    DownloadReceivedInvoices {
        #[command(flatten)]
        args: CrawlArgs,
        #[command(flatten)]
        filters: DownloadFilterArgs,
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
            CrawlCommands::ValidateCredentials { args } => {
                run_crawl_command(CrawlerType::ValidateCredentials, args, None).await;
            }
            CrawlCommands::DownloadInvoices { args, filters } => {
                run_crawl_command(CrawlerType::DownloadInvoices, args, Some(filters.into())).await;
            }
            CrawlCommands::DownloadIssuedInvoices { args, filters } => {
                run_crawl_command(
                    CrawlerType::DownloadIssuedInvoices,
                    args,
                    Some(filters.into()),
                )
                .await;
            }
            CrawlCommands::DownloadReceivedInvoices { args, filters } => {
                run_crawl_command(
                    CrawlerType::DownloadReceivedInvoices,
                    args,
                    Some(filters.into()),
                )
                .await;
            }
        },
    }
}
