use async_trait::async_trait;
use clap::{Parser, Subcommand};
use sat_cfdi;
use satcrawler::{
    get_download_folder, parse_date, Crawler, CrawlerConfig, CrawlerFilters, CrawlerOptions,
    CrawlerType, Credentials, Invoice, InvoiceEvent, InvoiceEventHandler, LoginType,
    SharedInvoiceEventHandler,
};
use std::env;
use std::io::{self, Write};
use std::sync::Arc;

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

struct CliInvoiceHandler {
    download_path: String,
}

impl CliInvoiceHandler {
    fn save_file(&self, uuid: &str, ext: &str, content: &[u8]) {
        let dest = std::path::Path::new(&self.download_path).join(format!("{}.{}", uuid, ext));
        if let Err(e) = std::fs::write(&dest, content) {
            eprintln!("[ERROR] Failed to save {}.{}: {}", uuid, ext, e);
        } else {
            eprintln!("[INFO] Saved {}", dest.display());
        }
    }
}

#[async_trait]
impl InvoiceEventHandler for CliInvoiceHandler {
    async fn should_download(&self, invoice: &Invoice) -> bool {
        let base = std::path::Path::new(&self.download_path);
        let xml_exists = base.join(format!("{}.xml", invoice.uuid)).exists();
        let pdf_exists = base.join(format!("{}.pdf", invoice.uuid)).exists();
        !xml_exists || !pdf_exists
    }

    async fn on_invoice_event(&self, event: InvoiceEvent) {
        match event {
            InvoiceEvent::XmlDownloaded { invoice, content } => {
                self.save_file(&invoice.uuid, "xml", &content);
            }
            InvoiceEvent::PdfDownloaded { invoice, content } => {
                self.save_file(&invoice.uuid, "pdf", &content);
            }
            InvoiceEvent::XmlDownloadFailed { invoice, error } => {
                eprintln!(
                    "[ERROR] XML download failed for {}: {}",
                    invoice.uuid, error
                );
            }
            InvoiceEvent::PdfDownloadFailed { invoice, error } => {
                eprintln!(
                    "[ERROR] PDF download failed for {}: {}",
                    invoice.uuid, error
                );
            }
            InvoiceEvent::Skipped { invoice } => {
                eprintln!("[INFO] Skipped {} (already exists)", invoice.uuid);
            }
        }
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
    let download_path = get_download_folder(Some(config.credentials.username.clone()));
    if let Err(e) = std::fs::create_dir_all(&download_path) {
        eprintln!(
            "[ERROR] Failed to create download directory {}: {}",
            download_path, e
        );
    }
    let handler: SharedInvoiceEventHandler = Arc::new(CliInvoiceHandler { download_path });
    let crawler = Crawler::new(crawler_type, config)
        .with_filters(filters)
        .with_event_handler(handler);
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
    /// Parse a CFDI XML file or all XML files in a directory and output JSON.
    /// A single file prints pretty JSON; a directory prints one compact JSON per line (NDJSON).
    Parse {
        /// Path to a .xml file or a directory containing .xml files.
        path: String,
        /// Output compact JSON instead of pretty-printed (always compact for directories).
        #[arg(long)]
        compact: bool,
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
        Commands::Parse { path, compact } => {
            let path = resolve_path(path);
            let p = std::path::Path::new(&path);
            if p.is_dir() {
                let entries = std::fs::read_dir(p).unwrap_or_else(|e| {
                    eprintln!("[ERROR] Cannot read directory {}: {}", path, e);
                    std::process::exit(1);
                });
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if file_path.extension().and_then(|e| e.to_str()) != Some("xml") {
                        continue;
                    }
                    let result = std::fs::read(&file_path)
                        .map_err(|e| e.to_string())
                        .and_then(|b| sat_cfdi::parse_bytes(&b).map_err(|e| e.to_string()))
                        .and_then(|inv| serde_json::to_string(&inv).map_err(|e| e.to_string()));
                    match result {
                        Ok(json) => println!("{}", json),
                        Err(e) => eprintln!("[ERROR] {}: {}", file_path.display(), e),
                    }
                }
            } else {
                let bytes = std::fs::read(&path).unwrap_or_else(|e| {
                    eprintln!("[ERROR] Cannot read {}: {}", path, e);
                    std::process::exit(1);
                });
                match sat_cfdi::parse_bytes(&bytes) {
                    Err(e) => {
                        eprintln!("[ERROR] {}", e);
                        std::process::exit(1);
                    }
                    Ok(invoice) => {
                        let json = if compact {
                            serde_json::to_string(&invoice)
                        } else {
                            serde_json::to_string_pretty(&invoice)
                        };
                        println!("{}", json.expect("JSON serialization error"));
                    }
                }
            }
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
