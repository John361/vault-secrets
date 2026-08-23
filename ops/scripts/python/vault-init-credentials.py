import argparse
import json
import os
import random
import string

from dotenv import load_dotenv
from pathlib import Path


def generate_password(length=20):
    characters = string.ascii_letters + string.digits
    return "".join(random.choice(characters) for _ in range(length))


def passwords_from_docker_env(app_name, path):
    raise ValueError("Not implemented")


def main():
    parser = argparse.ArgumentParser(description="Initialize Vault credentials")
    parser.add_argument("--app-name", required=True, help="App name")
    parser.add_argument("--environment", required=True, help="Environment name")
    args = parser.parse_args()

    script_base_dir = Path(__file__).resolve().parent
    docker_env_path = script_base_dir / ".." / ".." / "docker" / ".env"
    load_dotenv(dotenv_path=docker_env_path)

    input_file_name = f"../../terraform/utils/templates/vault-init-credentials.json"
    output_file_name = f"../../terraform/utils/json/vault-init-credentials-{args.environment}.json"

    input_file = os.path.join(os.path.dirname(__file__), input_file_name)
    output_file = os.path.join(os.path.dirname(__file__), output_file_name)

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
