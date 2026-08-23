# Vault Secrets

## Start the dev environment
```shell
cd ops/docker

mkdir -p data/vault
sudo chown root:root -R data/vault && sudo chmod 777 -R data/vault # Fix for error when starting docker container

docker compose up --build # Access and configure Vault from the web ui (use only 1 key for simplicity)
```
