import json
import os
import runpy
from unittest import mock

import pytest

import vault_init_credentials as vic


def test_generate_password_length():
    pwd = vic.generate_password(length=30)
    assert len(pwd) == 30
    assert isinstance(pwd, str)


def test_passwords_from_docker_env_postgres():
    with mock.patch.dict(os.environ, {"POSTGRES_PASSWORD": "postgres_pwd"}):
        assert (
            vic.passwords_from_docker_env("app", "postgres/users/admin")
            == "postgres_pwd"
        )


def test_passwords_from_docker_env_terraform():
    with mock.patch.dict(os.environ, {"TERRAFORM_DB_PASSWORD": "tf_pwd"}):
        assert (
            vic.passwords_from_docker_env("app", "postgres/users/terraform") == "tf_pwd"
        )


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
        {"path": "unknown/path", "data": {"password": "old_pwd"}},
    ]
    input_file.write_text(json.dumps(data))

    monkeypatch.setattr(vic, "load_dotenv", lambda **kwargs: None)
    monkeypatch.setattr(vic, "script_base_dir", tmp_path)
    monkeypatch.setattr(vic, "base_dir", base_dir.resolve())
    monkeypatch.setattr(vic, "input_file", input_file.resolve())

    mock_args = mock.Mock()
    mock_args.app_name = "test-app"
    mock_args.environment = "dev"
    mock_parser = mock.Mock()
    mock_parser.parse_args.return_value = mock_args
    monkeypatch.setattr(vic.argparse, "ArgumentParser", lambda **kwargs: mock_parser)

    return tmp_path, base_dir, input_file, json_dir


def test_main_input_path_invalid(tmp_path, monkeypatch):
    monkeypatch.setattr(vic, "base_dir", tmp_path.resolve())
    monkeypatch.setattr(vic, "input_file", tmp_path.parent / "outside.json")
    monkeypatch.setattr(
        "sys.argv",
        ["vault_init_credentials.py", "--app-name", "test-app", "--environment", "dev"],
    )

    with pytest.raises(ValueError):
        vic.main()


def test_main_output_path_invalid(tmp_path, monkeypatch):
    monkeypatch.setattr(vic, "base_dir", tmp_path.resolve())
    monkeypatch.setattr(
        "sys.argv",
        [
            "vault_init_credentials.py",
            "--app-name",
            "test-app",
            "--environment",
            "../../../../etc/passwd",
        ],
    )

    valid_input = (
        tmp_path / "terraform" / "utils" / "templates" / "vault-init-credentials.json"
    )
    valid_input.parent.mkdir(parents=True)
    valid_input.write_text(json.dumps([{"path": "x", "data": {"password": "pwd"}}]))
    monkeypatch.setattr(vic, "input_file", valid_input.resolve())

    with pytest.raises(ValueError):
        vic.main()


def test_main_happy_path_with_env(setup_dirs):
    _tmp_path, _base_dir, _input_file, json_dir = setup_dirs

    with mock.patch.dict(os.environ, {"POSTGRES_PASSWORD": "super_secret_pwd"}):
        vic.main()

    output_file = json_dir / "vault-init-credentials-dev.json"
    assert output_file.exists()

    data = json.loads(output_file.read_text())
    assert data[0]["data"]["password"] == "super_secret_pwd"
    assert data[1]["data"]["password"] != "old_pwd"


def test_main_fallback_to_generated_password(setup_dirs):
    _tmp_path, _base_dir, _input_file, json_dir = setup_dirs

    os.environ.pop("POSTGRES_PASSWORD", None)

    vic.main()

    output_file = json_dir / "vault-init-credentials-dev.json"
    data = json.loads(output_file.read_text())

    assert data[0]["data"]["password"] is None
    assert len(data[1]["data"]["password"]) == 20


def test_run_as_main():
    with (
        mock.patch("sys.argv", ["vault_init_credentials.py"]),
        pytest.raises(SystemExit),
    ):
        runpy.run_module("vault_init_credentials", run_name="__main__")
