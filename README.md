# SAT CLI
Open Source project to crawl over the Mexican Tax Service.

# WARNING
This tool is in active development, so there's just a lot of things either not defined or not working
completely, if you want to collaborate feel free to reach out or open a PR :)

## Installation

```bash
git clone https://github.com/carlosforcss/sat-cli.git
cd sat-cli
cargo build --release
cp target/release/sat-cli /usr/local/bin/sat-cli
```

## Usage

### Set up 
```
export TWOCAPTCHA_API_TOKEN=your_2captcha_api_token
export DOWNLOAD_FOLDER=your_absolute_download_path (relative /downloads by default) 
```

### Current working commands
```bash
sat-cli validate_credentials RFCXXXX PASSXXXXX
// Returns if the CIEC credentials are valid or not
```

```bash
sat-cli download_invoices RFCXXXX PASSXXXX
Download the invoices for the given CIEC credentials in the /downloads folder from your execution
```

