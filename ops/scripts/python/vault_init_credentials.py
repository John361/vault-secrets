import argparse
import json
import os
import secrets
import string

from dotenv import load_dotenv
from pathlib import Path


script_base_dir = Path(__file__).resolve().parent
base_dir = Path(__file__).resolve().parent.parent.parent / "terraform" / "utils"
input_file = (base_dir / "templates" / "vault-init-credentials.json").resolve()


def generate_password(length=20):
    characters = string.ascii_letters + string.digits
    return "".join(secrets.choice(characters) for _ in range(length))


def passwords_from_docker_env(app_name, path):
    if path == "postgres/users/admin":
        return os.environ.get("POSTGRES_PASSWORD")
    elif path == "postgres/users/terraform":
        return os.environ.get("TERRAFORM_DB_PASSWORD")
    else:
        raise ValueError("Path not found")


def main():
    parser = argparse.ArgumentParser(description="Initialize Vault credentials")
    parser.add_argument("--app-name", required=True, help="App name")
    parser.add_argument("--environment", required=True, help="Environment name")
    args = parser.parse_args()

    docker_env_path = script_base_dir / ".." / ".." / "docker" / ".env"
    load_dotenv(dotenv_path=docker_env_path)

    output_file = (base_dir / "json" / f"vault-init-credentials-{args.environment}.json").resolve()

    if not str(input_file).startswith(str(base_dir.resolve())):
        raise ValueError("Invalid input path")
    if not str(output_file).startswith(str(base_dir.resolve())):
        raise ValueError("Invalid output path")

    output_file.parent.mkdir(parents=True, exist_ok=True)

    with open(input_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    for entry in data:
        if "data" in entry and "password" in entry["data"]:
            try:
                entry["data"]["password"] = passwords_from_docker_env(args.app_name, entry["path"])
            except ValueError:
                entry["data"]["password"] = generate_password()

    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=4, ensure_ascii=False)


if __name__ == "__main__":
    main()
