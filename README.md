# Vault Secrets

## Requirements
```shell
curl -LsSf https://astral.sh/uv/install.sh | sh

brew tap hashicorp/tap
brew install hashicorp/tap/terraform
brew install terragrunt
```

## Start the dev environment
```shell
# Docker
cd ops/docker

mkdir -p data/vault
sudo chown root:root -R data/vault && sudo chmod 777 -R data/vault # Fix for error when starting docker container

docker compose up --build # Access and configure Vault from the web ui (use only 1 key for simplicity)

# Python scripts
cd ops/scripts/python
uv sync
source .venv/bin/activate

python vault-init-credentials.py --app-name vault-secrets --environment dev
```
