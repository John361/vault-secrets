# Vault Secrets
Vault Secrets is a simple Rust CLI app that retrieves a secret from a Vault instance and displays it encoded in base64.

## Requirements
```shell
curl -LsSf https://astral.sh/uv/install.sh | sh

brew tap hashicorp/tap
brew install hashicorp/tap/terraform
brew install terragrunt

rustup update
cargo install cargo-deb
```

## Start the dev environment
```shell
# Docker
cd ops/docker

mkdir -p data/vault
sudo chown root:root -R data/vault && sudo chmod 777 -R data/vault # Fix for error when starting docker container

cp .env.template .env # Then replace all 'changeme' values (check on helpers section for random password generation)

docker compose up --build # Access and configure Vault from the web ui (use only 1 key for simplicity)

# Python scripts
cd ops/scripts/python
uv sync
source .venv/bin/activate

python vault-init-credentials.py --app-name vault-secrets --environment dev

# Terraform / Terragrunt
echo "postgresql://terraform:changeme@127.0.0.1:5432/terraform?sslmode=disable" >> ops/terraform/.postgres_backend_uri_dev

cd ops/terraform/modules/dev
export VAULT_TOKEN="changeme"

cd vault/secret-engine-kv
terragrunt plan && terragrunt apply --auto-approve

cd vault/init-credentials
terragrunt plan && terragrunt apply --auto-approve

cd vault/auth-backend-userpass
terragrunt plan && terragrunt apply --auto-approve

# Rust
cp app.conf.template.yml app.conf.yml # # Then replace all 'changeme' values

RUST_LOG=debug cargo run -- --config app.conf.yml find --path "vault/users/vault-secrets" --key "username"
```

## Build app
```shell
cargo build --release
cargo deb
```

## Installation
### Automatically using Ansible
[GitHub Project](https://github.com/John361/ansible-vault-secrets)

### Manually using shell
```shell
curl -fsSL https://john361.github.io/vault-secrets/vault-secrets.gpg | sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/vault-secrets.gpg
cat << EOF > /etc/apt/sources.list.d/vault-secrets.sources 
Types: deb
URIs: https://john361.github.io/vault-secrets/
Suites: stable
Components: main
Signed-By: /etc/apt/trusted.gpg.d/vault-secrets.gpg
EOF

sudo apt update
sudo apt install vault-secrets

cat /etc/vault-secrets/app.conf.yml # And replace information
```

## Helpers
- Generate a random password: `tr -dc A-Za-z0-9 </dev/urandom | head -c "20" ; echo ''`
