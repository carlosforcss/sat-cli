use clap::{Parser, Subcommand};
use satcrawler::{Crawler, CrawlerConfig, CrawlerType};
use std::env;

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
    Docs,
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

            match env::var("SATCLI_DOCUMENTS_FOLDER") {
                Ok(path) => println!("  [ok] Custom documents folder: {}", path),
                Err(_) => {
                    println!("  [-]  No custom documents folder set — using ~/sat-cli/documents/")
                }
            }
        }
        Commands::Docs => {
            println!("sat-cli — available commands:\n");
            println!(
                "  crawl validate-credentials [--username <USERNAME>] [--password <PASSWORD>]"
            );
            println!("      Validates SAT credentials by attempting a login.");
            println!();
            println!("  crawl download-invoices [--username <USERNAME>] [--password <PASSWORD>]");
            println!("      Downloads all issued and received invoices from the SAT portal.");
            println!();
            println!("Credentials are read from ~/sat-cli/config.json when flags are omitted.");
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
