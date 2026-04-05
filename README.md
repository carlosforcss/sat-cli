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

## Configuration

```bash
export TWOCAPTCHA_API_KEY=your_2captcha_api_key
export SATCLI_DOCUMENTS_FOLDER=/absolute/path/to/downloads  # optional, defaults to ~/sat-cli/documents/
```

Credentials can also be persisted in `~/sat-cli/config.json` and omitted from commands.

## Commands

```bash
sat-cli docs
# Lists all available commands

sat-cli doctor
# Checks environment configuration and reports any issues

sat-cli crawl validate-credentials --username RFCXXXX --password PASSXXXXX
# Returns whether the CIEC credentials are valid

sat-cli crawl download-invoices --username RFCXXXX --password PASSXXXXX
# Downloads all issued and received invoices to the documents folder
```
