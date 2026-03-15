use std::env;
use crawler::Crawler;

#[derive(Debug)]
enum Command {
    Help,
    ValidateCredentials {
        username: String,
        password: String
    } 
}


impl Command {
    fn get() -> Result<Command, String> {
        let mut args = env::args();
        args.next(); // Skip the first argument wich is the program name
        
        let main_arg = args.next()
            .expect("No argument inserted");
        
        let command = match main_arg.as_str() {
            "validate_credentials" => {
                let username = args.next()
                    .expect("No username provided");
                let password = args.next()
                    .expect("No password provided");
                Command::ValidateCredentials { 
                    username: username,
                    password: password
                }
            },
            "help" => Command::Help,
            _ => Command::Help,
        };

        Ok(command) 
    }

    async fn run(&self) {
        match &self {
            Command::Help => println!("Executing help command"),
            Command::ValidateCredentials { username, password } => {
                Crawler::ValidateCredentials {
                    username: username.clone(),
                    password: password.clone(),
                }.run().await.expect("ERr running crawler");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Hello world! :)");
    let command = Command::get()
        .expect("Error getting command");
    dbg!("Running {} comand", &command);
    command.run().await;
}
