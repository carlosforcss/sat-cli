# SAT CLI
Open Source project to crawl over the Mexican Tax Service.

> [!WARNING]
> This tool is in active development, so there's just a lot of things either not defined or not working
> completely. If you want to collaborate feel free to reach out or open a PR :)

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

Run `sat-cli config` to save credentials interactively. Saved credentials are loaded automatically
and can be overridden per-command with flags.

## Commands

### doctor
```bash
sat-cli doctor
```
Checks environment configuration and reports any issues (API key, credentials, FIEL certificate paths).

### config
```bash
sat-cli config
```
Interactive prompt to save credentials (RFC, password, login type, FIEL certificate paths) to `~/sat-cli/config.json`.

### crawl validate-credentials
```bash
sat-cli crawl validate-credentials
sat-cli crawl validate-credentials --username RFCXXXX --password PASSXXXXX
sat-cli crawl validate-credentials --username RFCXXXX --password PASSXXXXX --login-type fiel --crt ~/certs/cert.cer --key ~/certs/key.key
```
Validates that the configured credentials can log in to the SAT portal. Run `sat-cli config` first
to save credentials — including FIEL certificate paths for FIEL login.

### crawl download-invoices
```bash
sat-cli crawl download-invoices
sat-cli crawl download-invoices --start-date 01/01/2024 --end-date 31/12/2024
```
Downloads all issued and received invoices to the documents folder.

### crawl download-issued-invoices
```bash
sat-cli crawl download-issued-invoices
sat-cli crawl download-issued-invoices --start-date 01/01/2025
```
Downloads only issued invoices.

### crawl download-received-invoices
```bash
sat-cli crawl download-received-invoices
sat-cli crawl download-received-invoices --start-date 01/01/2025 --end-date 31/03/2025
```
Downloads only received invoices.

## Common flags

All `crawl` subcommands accept:

| Flag | Description |
|---|---|
| `--username` | RFC (overrides config) |
| `--password` | Password (overrides config) |
| `--login-type` | `ciec` or `fiel` (overrides config) |
| `--crt` | Path to FIEL certificate `.cer` (overrides config) |
| `--key` | Path to FIEL private key `.key` (overrides config) |
| `--headless` | Run browser in headless mode |
| `--sandbox` | Run browser with sandbox enabled |

Download commands additionally accept:

| Flag | Format | Description |
|---|---|---|
| `--start-date` | `dd/mm/YYYY` | Skip invoices issued before this date |
| `--end-date` | `dd/mm/YYYY` | Skip invoices issued after this date |
