import json
import os
import pytest
import vault_init_credentials as vic

from unittest import mock
from pathlib import Path


def test_generate_password_length():
    pwd = vic.generate_password(length=30)
    assert len(pwd) == 30
    assert isinstance(pwd, str)


def test_passwords_from_docker_env_postgres():
    with mock.patch.dict(os.environ, {"POSTGRES_PASSWORD": "postgres_pwd"}):
        assert vic.passwords_from_docker_env("app", "postgres/users/admin") == "postgres_pwd"


def test_passwords_from_docker_env_terraform():
    with mock.patch.dict(os.environ, {"TERRAFORM_DB_PASSWORD": "tf_pwd"}):
        assert vic.passwords_from_docker_env("app", "postgres/users/terraform") == "tf_pwd"


def test_passwords_from_docker_env_unknown_path():
    with pytest.raises(ValueError):
        vic.passwords_from_docker_env("app", "unknown/path")


@pytest.fixture
def setup_dirs(tmp_path, monkeypatch):
    base_dir = tmp_path / "terraform" / "utils"
    templates_dir = base_dir / "templates"
    json_dir = base_dir / "json"
    templates_dir.mkdir(parents=True)
    json_dir.mkdir(parents=True)

    input_file = templates_dir / "vault-init-credentials.json"
    data = [
        {"path": "postgres/users/admin", "data": {"password": "old_pwd"}},
        {"path": "unknown/path", "data": {"password": "old_pwd"}}
    ]
    input_file.write_text(json.dumps(data))

    monkeypatch.setattr(vic, 'load_dotenv', lambda **kwargs: None)
    monkeypatch.setattr(vic, 'script_base_dir', tmp_path)
    monkeypatch.setattr(vic, 'base_dir', base_dir.resolve())

    mock_args = mock.Mock()
    mock_args.app_name = "test-app"
    mock_args.environment = "dev"
    monkeypatch.setattr(vic.argparse, 'ArgumentParser', lambda **kwargs: mock.Mock(parse_args=lambda: mock_args))

    return tmp_path, base_dir, input_file, json_dir


def test_main_input_path_invalid(tmp_path, monkeypatch):
    monkeypatch.setattr(vic, 'base_dir', tmp_path.resolve())
    outside_file = tmp_path / ".." / "outside.json"
    monkeypatch.setattr(Path, 'resolve', lambda self: outside_file.resolve())

    with pytest.raises(ValueError):
        vic.main()


def test_main_output_path_invalid(tmp_path, monkeypatch):
    monkeypatch.setattr(vic, 'base_dir', tmp_path.resolve())

    with mock.patch.object(vic.argparse, 'ArgumentParser') as mock_parser:
        mock_args = mock.Mock()
        mock_args.app_name = "app"
        mock_args.environment = "dev"
        mock_parser.return_value.parse_args.return_value = mock_args

        with mock.patch.object(Path, 'resolve', side_effect=lambda self: self):
            vic.base_dir = tmp_path
            mock_input = tmp_path / "in.json"
            mock_input.touch()
            mock_output = tmp_path.parent / "out.json"

            with mock.patch.object(vic, 'input_file', mock_input), \
                    mock.patch.object(vic, 'output_file', mock_output):
                with pytest.raises(ValueError):
                    vic.main()


def test_main_happy_path_with_env(setup_dirs, monkeypatch):
    tmp_path, base_dir, input_file, json_dir = setup_dirs

    monkeypatch.setenv("POSTGRES_PASSWORD", "super_secret_pwd")

    vic.main()

    output_file = json_dir / "vault-init-credentials-dev.json"
    assert output_file.exists()

    data = json.loads(output_file.read_text())
    assert data[0]["data"]["password"] == "super_secret_pwd"
    assert data[1]["data"]["password"] != "old_pwd"

def test_main_fallback_to_generated_password(setup_dirs, monkeypatch):
    tmp_path, base_dir, input_file, json_dir = setup_dirs

    monkeypatch.delenv("POSTGRES_PASSWORD", raising=False)

    vic.main()

    output_file = json_dir / "vault-init-credentials-dev.json"
    data = json.loads(output_file.read_text())

    assert len(data[0]["data"]["password"]) == 20
