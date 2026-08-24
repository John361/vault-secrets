[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=John361_vault-secrets&metric=alert_status)](https://sonarcloud.io/dashboard?id=John361_vault-secrets)

# Vault Secrets
A secure, lightweight Rust CLI application designed to fetch secrets from Hashicorp Vault and output them in base64 format.

This tool is specifically built for server automation scripts, allowing server administrators to securely retrieve runtime secrets without hardcoding credentials on the host system.

## Overview
Vault Secrets is designed for server-side scripting scenarios where storing plaintext credentials is not an option. Instead of hardcoding secrets in your scripts or configuration files, you can retrieve them on-demand from HashiCorp Vault at runtime.

Key Features:
- **Secure Retrieval:** Interacts directly with Hashicorp Vault to fetch sensitive data dynamically
- **Base64 Encoding:** Outputs secrets in base64 format for seamless pipeline and script integration
- **Debian Packaging:** Fully integrated with GitHub Actions to automatically build and publish `.deb` packages for easy management via `apt`
- **Open Source:** Distributed under the terms of the GNU Affero General Public License v3 (AGPLv3)
- **Reliability:** Lightweight and fast, written in Rust
- **Integration:** Integrates seamlessly with existing automation and CI/CD pipelines

## Installation
### Debian/Ubuntu Repository (Recommended)
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

### Ansible Automation
For automated deployments across multiple servers, use our official Ansible role:

[ansible-vault-secrets](https://github.com/John361/ansible-vault-secrets) - Manages installation, configuration, and updates.

### Manual binary installation
Download the latest `.deb` package from the [Releases](https://github.com/John361/vault-secrets/releases) page and install it manually:
```shell
sudo apt install ./vault-secrets_<version>_amd64.deb

sudo mkdir /etc/vault-secrets && sudo chown root:root -R /etc/vault-secrets
sudo nano /etc/vault-secrets/app.conf.yml
sudo chmod 400 /etc/vault-secrets/app.conf.yml
```

## Development environment
### Prerequisites
```shell
curl -LsSf https://astral.sh/uv/install.sh | sh

brew tap hashicorp/tap
brew install hashicorp/tap/terraform
brew install terragrunt

rustup update
cargo install cargo-deb
```

### Starting the local development stack
#### Docker
```shell
cd ops/docker

mkdir -p data/vault
sudo chown root:root -R data/vault && sudo chmod 777 -R data/vault # Fix for error when starting docker container

cp .env.template .env # Then replace all 'changeme' values (check on helpers section for random password generation)

docker compose up --build # Access and configure Vault from the web ui (use only 1 key for simplicity)
```

#### Initialize Vault Credentials with Python
```shell
cd ops/scripts/python
uv sync
source .venv/bin/activate

python vault-init-credentials.py --app-name vault-secrets --environment dev
```

#### Provision Secrets with Terraform/Terragrunt
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

#### Test the Rust application
```shell
cp app.conf.template.yml app.conf.yml # # Then replace all 'changeme' values

RUST_LOG=debug cargo run -- --config app.conf.yml find --path "vault/users/vault-secrets" --key "username"
```

## Building from source
### Release binary
```shell
cargo build --release
```

### Debian package
```shell
cargo deb
```
The `.deb` package will be generated in `target/debian/` folder.

## Usage
### Command syntax
```shell
vault-secrets --config <CONFIG_FILE> find --path <SECRET_PATH> --key <SECRET_KEY>
```

### Example
```shell
vault-secrets --config /etc/vault-secrets/app.conf.yml find --path "vault/users/my-app" --key "api_key"
# Output: base64-encoded secret
```

### Using in Shell Scripts
```shell
#!/bin/bash

API_KEY=$(vault-secrets --config /etc/vault-secrets/app.conf.yml find --path "vault/services/api" --key "key" | base64 -d)
curl -H "Authorization: Bearer ${API_KEY}" https://api.example.com/data
```

## Helpers
- Generate a random password: `tr -dc A-Za-z0-9 </dev/urandom | head -c "20" ; echo ''`

## License
This project is licensed under the GNU Affero General Public License v3.0 - see the LICENSE file for details.
This license requires that modifications to this software, even when used over a network, must be made available to the users. For more information, see the GNU AGPLv3 documentation.
