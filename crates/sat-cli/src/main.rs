use satcrawler::{Crawler, CrawlerConfig, CrawlerOptions, Credentials};
use std::env;

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
        let options = CrawlerOptions::default();
        match &self {
            Command::Help => println!("Executing help command"),
            Command::ValidateCredentials { username, password } => {
                let config = CrawlerConfig::new(
                    Credentials {
                        username: username.clone(),
                        password: password.clone(),
                    },
                    options
                );
                let response = Crawler::ValidateCredentials(config).run().await
                .expect("Err running crawler");
                dbg!(response);
            }
            Command::DownloadInvoices { username, password } => {
                 let config = CrawlerConfig::new(
                    Credentials {
                        username: username.clone(),
                        password: password.clone(),
                    },
                    options
                );
                let response = Crawler::DownloadInvoices(config).run().await
                    .expect("Err running crawler");
                dbg!(response);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Hello world! :)");
    let command = Command::get().expect("Error getting command");
    dbg!("Running {} comand", &command);
    command.run().await;
}
