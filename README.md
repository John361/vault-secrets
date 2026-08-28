<img src="ops/assets/icon.jpeg" alt="" />

[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=John361_vault-secrets&metric=alert_status)](https://sonarcloud.io/dashboard?id=John361_vault-secrets)
[![Security Rating](https://sonarcloud.io/api/project_badges/measure?project=John361_vault-secrets&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=John361_vault-secrets)
[![software_quality_security_issues](https://sonarcloud.io/api/project_badges/measure?project=John361_vault-secrets&metric=software_quality_security_issues)](https://sonarcloud.io/summary/new_code?id=John361_vault-secrets)
[![Vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=John361_vault-secrets&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=John361_vault-secrets)

# Vault Secrets
A secure, lightweight Rust CLI application designed to fetch secrets from Hashicorp Vault and output them in base64 format (unless the clear output option is provided).

This tool is specifically built for server automation scripts, allowing server administrators to securely retrieve runtime secrets without hardcoding credentials on the host system.

Vault Secrets can also be used to export or import secrets from and to a Hashicorp Vault instances. This is useful for extracted backups.

## Overview
Vault Secrets is originally designed for server-side scripting scenarios where storing plaintext credentials is not an option. Instead of hardcoding secrets in your scripts or configuration files, you can retrieve them on-demand from HashiCorp Vault at runtime.

For personal convenient, it is always designed for full encrypted secrets exports and imports secrets backups.

Key Features:
- :lock: **Secure Retrieval:** Interacts directly with Hashicorp Vault to fetch sensitive data dynamically
- :file_folder: **Base64 Encoding:** Outputs secrets in base64 format (unless the clear output configuration is provided) for seamless pipeline and script integration
- :outbox_tray: **Data export:** Export your data from your instance to a dedicated file with default managed encryption
- :inbox_tray: **Data import:** Import your data to your instance from a dedicated file with default managed encryption
- :alarm_clock: **Rate limiting:** Configure a sleep option to wait between each call to the Vault instance
- :package: **Debian Packaging:** Fully integrated with GitHub Actions to automatically build and publish `.deb` packages for easy management via `apt`
- :scroll: **Open Source:** Distributed under the terms of the GNU Affero General Public License v3 (AGPLv3)
- :crab: **Reliability:** Lightweight and fast, written in Rust
- :arrows_counterclockwise: **Continuous Integration:** Automated linting, testing, and formatting checks on every pull request to ensure code quality and prevent regressions
- :link: **Integration:** Integrates seamlessly with existing automation and CI/CD pipelines
- :shield: **Security:** Continuous code quality and vulnerability scanning integrated via SonarCloud

## Installation
### :package: Debian/Ubuntu Repository (Recommended)
Add the official repository and install the package:
```shell
# Add the GPG key
curl -fsSL https://john361.github.io/vault-secrets/vault-secrets.gpg | sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/vault-secrets.gpg

# Add the repository
cat << EOF | sudo tee /etc/apt/sources.list.d/vault-secrets.sources
Types: deb
URIs: https://john361.github.io/vault-secrets/
Suites: stable
Components: main
Signed-By: /etc/apt/trusted.gpg.d/vault-secrets.gpg
EOF

# Install
sudo apt update
sudo apt install vault-secrets
```
After installation, configure the app by editing `/etc/vault-secrets/app.conf.yml` with your Vault connection details.

### :magic_wand: Ansible Automation
For automated deployments across multiple servers, use our official Ansible role:

[ansible-vault-secrets](https://github.com/John361/ansible-vault-secrets) - Manages installation, configuration, and updates.

### :floppy_disk: Manual binary installation
Download the latest `.deb` package from the [Releases](https://github.com/John361/vault-secrets/releases) page and install it manually:
```shell
sudo apt install ./vault-secrets_<version>_amd64.deb

sudo mkdir /etc/vault-secrets && sudo chown root:root -R /etc/vault-secrets
sudo nano /etc/vault-secrets/app.conf.yml
sudo chmod 400 /etc/vault-secrets/app.conf.yml
```

## :test_tube: Development environment
### :clipboard: Prerequisites
```shell
curl -LsSf https://astral.sh/uv/install.sh | sh

brew tap hashicorp/tap
brew install hashicorp/tap/terraform
brew install terragrunt
brew install terraform-linters/tap/tflint

rustup update
cargo install cargo-deb
```

### :rocket: Starting the local development stack
#### :whale: Docker
```shell
cd ops/docker

mkdir -p data/vault
sudo chown root:root -R data/vault && sudo chmod 777 -R data/vault # Fix for error when starting docker container

cp .env.template .env # Then replace all 'changeme' values (check on helpers section for random password generation)

docker compose up --build # Access and configure Vault from the web ui (use only 1 key for simplicity)
```

#### :snake: Initialize Vault Credentials with Python
```shell
cd ops/scripts/python
uv sync
source .venv/bin/activate

python vault_init_credentials.py --app-name vault-secrets --environment dev
```

#### :building_construction: Provision Secrets with Terraform/Terragrunt
```shell
echo "postgresql://terraform:changeme@127.0.0.1:5432/terraform?sslmode=disable" >> ops/terraform/.postgres_backend_uri_dev

cd ops/terraform/modules/dev
export VAULT_TOKEN="changeme"

cd vault/secret-engine-kv
terragrunt plan && terragrunt apply --auto-approve

cd vault/init-credentials
terragrunt plan && terragrunt apply --auto-approve

cd vault/auth-backend-userpass
terragrunt plan && terragrunt apply --auto-approve
```

#### :crab: Test the Rust application
```shell
cp app.conf.template.yml app.conf.yml # # Then replace all 'changeme' values

RUST_LOG=debug cargo run -- --config app.conf.yml --help
```

#### :mag: Run linters
```shell
# Rust
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Terraform
cd ops/terraform/libs
tflint --config=$(pwd)/.tflint.hcl --recursive --format=default

# Terragrunt
cd ops/terraform/modules
terragrunt hcl fmt --check

# Python
cd ops/scripts/python
uv run --frozen ruff check --output-format=github .
uv run --frozen ruff format --check
```

#### :white_check_mark: Run tests
```shell
# Rust
cargo test

# Python
cd ops/scripts/python
uv run --frozen pytest --cov=vault_init_credentials --cov-report=term-missing --cov-report=xml
```

## :wrench: Building from source
### :package: Release binary
```shell
cargo build --release
```

### :package: Debian package
```shell
cargo deb
```
The `.deb` package will be generated in `target/debian/` folder.

## :book: Usage
### :mag: Find secrets
```shell
# Basic usage
vault-secrets --config <CONFIG_FILE> find --mount <MOUNT_PATH> --path <SECRET_PATH> --key <SECRET_KEY>
```

### :outbox_tray: Export secrets
```shell
# Basic usage
vault-secrets --config <CONFIG_FILE> export --path <PATH> --output-folder <OUTPUT_FOLDER>
```

### :inbox_tray: Import secrets
```shell
# Basic usage
vault-secrets --config <CONFIG_FILE> import --path <PATH> --input-folder <OUTPUT_FOLDER>
```

### :bulb: Example
```shell
vault-secrets --config /etc/vault-secrets/app.conf.yml find --mount "secret-v2" --path "vault/users/my-app" --key "api_key"
vault-secrets --config /etc/vault-secrets/app.conf.yml find --mount "secret-v1" --path "vault/users/my-app" --key "api_key" --engine "kv1"
# Output: base64-encoded secret

vault-secrets --config /etc/vault-secrets/app.conf.yml export --path "" --output-folder "./tests"
# Output: in file

vault-secrets --config /etc/vault-secrets/app.conf.yml input --path "" --input-folder "./tests"
# Output: in your instance
```

### :scroll: Using in Shell Scripts
#### :calendar: Example for cron jobs
```shell
#!/bin/bash

API_KEY=$(vault-secrets --config /etc/vault-secrets/app.conf.yml find --mount "secret-v2" --path "vault/services/api" --key "key" | base64 -d)
curl -H "Authorization: Bearer ${API_KEY}" https://api.example.com/data
```

#### :calendar: :outbox_tray: Example for export cron jobs
```shell
#!/bin/bash

today="$(date +%Y-%m-%d)"
backup_path="/path/to/backups/${today}"

vault-secrets --config /etc/vault-secrets/app.conf.yml export --path "" --output-folder "${backup_file}"
```

#### :calendar: :inbox_tray: Example for import cron jobs
```shell
#!/bin/bash

today="$(date +%Y-%m-%d)"
backup_path="/path/to/backups/${today}"

vault-secrets --config /etc/vault-secrets/app.conf.yml import --path "" --input-folder "${backup_file}"
```

## :hammer_and_wrench: Helpers
- Generate a random password: `tr -dc A-Za-z0-9 </dev/urandom | head -c "20" ; echo ''`

## :scroll: License
This project is licensed under the GNU Affero General Public License v3.0 - see the LICENSE file for details.
This license requires that modifications to this software, even when used over a network, must be made available to the users. For more information, see the GNU AGPLv3 documentation.
