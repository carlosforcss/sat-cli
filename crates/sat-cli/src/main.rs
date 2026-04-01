use satcrawler::{Crawler, CrawlerConfig, CrawlerOptions, CrawlerType, Credentials};
use std::env;
use tracing::info;

#[derive(Debug)]
enum Command {
    Help,
    ValidateCredentials { username: String, password: String },
    DownloadInvoices { username: String, password: String },
}

impl Command {
    fn get() -> Result<Command, String> {
        let mut args = env::args();
        args.next(); // Skip the first argument wich is the program name

        let main_arg = args.next().expect("No argument inserted");

        let command = match main_arg.as_str() {
            "validate_credentials" => {
                let username = args.next().expect("No username provided");
                let password = args.next().expect("No password provided");
                Command::ValidateCredentials {
                    username: username,
                    password: password,
                }
            }
            "download_invoices" => {
                let username = args.next().expect("No username provided");
                let password = args.next().expect("No password provided");
                Command::DownloadInvoices {
                    username: username,
                    password: password,
                }
            }
            "help" => Command::Help,
            _ => Command::Help,
        };

        Ok(command)
    }

    async fn run(&self) {
        let mut config = CrawlerConfig::new_from_file();
        match &self {
            Command::Help => println!("Executing help command"),
            Command::ValidateCredentials { username, password } => {
                config.credentials.username = username.clone();
                config.credentials.password = password.clone();
                let crawler = Crawler::new(CrawlerType::ValidateCredentials, config);
                let response = crawler.run().await;
                println!(
                    "{}",
                    serde_json::to_string(&response).expect("Response serialization error")
                );
            }
            Command::DownloadInvoices { username, password } => {
                config.credentials.username = username.clone();
                config.credentials.password = password.clone();
                let crawler = Crawler::new(CrawlerType::DownloadInvoices, config);
                let response = crawler.run().await;
                println!(
                    "{}",
                    serde_json::to_string(&response).expect("Response serialization error")
                );
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let command = Command::get().expect("Error getting command");
    info!("Runnign command {:?}", &command);
    command.run().await;
}
