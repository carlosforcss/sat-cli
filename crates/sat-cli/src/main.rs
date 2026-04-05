use clap::{Parser, Subcommand};
use satcrawler::{Crawler, CrawlerConfig, CrawlerType};

#[derive(Parser)]
#[command(name = "sat-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        Commands::ValidateCredentials { username, password } => {
            let mut config = CrawlerConfig::new_from_file();
            if let Some(u) = username {
                config.credentials.username = u;
            }
            if let Some(p) = password {
                config.credentials.password = p;
            }
            let crawler = Crawler::new(CrawlerType::ValidateCredentials, config);
            let response = crawler.run().await;
            println!(
                "{}",
                serde_json::to_string(&response).expect("Response serialization error")
            );
        }
        Commands::DownloadInvoices { username, password } => {
            let mut config = CrawlerConfig::new_from_file();
            if let Some(u) = username {
                config.credentials.username = u;
            }
            if let Some(p) = password {
                config.credentials.password = p;
            }
            let crawler = Crawler::new(CrawlerType::DownloadInvoices, config);
            let response = crawler.run().await;
            println!(
                "{}",
                serde_json::to_string(&response).expect("Response serialization error")
            );
        }
    }
}
