use assert_cmd::Command;
use std::io::Write;

fn temp_config() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut file = std::fs::File::create(&path).unwrap();
    write!(
        file,
        r#"server_url = "http://127.0.0.1:9"
websocket_url = "ws://127.0.0.1:9"
"#
    )
    .unwrap();
    (dir, path)
}

#[test]
fn webhook_set_parses_url() {
    let (_dir, config_path) = temp_config();
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--config",
        config_path.to_str().unwrap(),
        "webhook",
        "set",
        "http://localhost:3000/hooks/agentlink",
    ]);

    cmd.assert().success();
}

#[test]
fn webhook_get_shows_status() {
    let (_dir, config_path) = temp_config();
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--config",
        config_path.to_str().unwrap(),
        "webhook",
        "get",
    ]);

    cmd.assert().success();
}

#[test]
fn webhook_delete_removes_config() {
    let (_dir, config_path) = temp_config();
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--config",
        config_path.to_str().unwrap(),
        "webhook",
        "delete",
    ]);

    cmd.assert().success();
}
