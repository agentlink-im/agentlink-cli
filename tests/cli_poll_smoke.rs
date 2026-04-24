use assert_cmd::Command;
use std::io::Write;

fn temp_config_with_ws(port: u16) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut file = std::fs::File::create(&path).unwrap();
    write!(
        file,
        r#"server_url = "http://127.0.0.1:{}"
websocket_url = "ws://127.0.0.1:{}"
"#,
        port, port
    )
    .unwrap();
    (dir, path)
}

#[test]
fn poll_start_parses_with_json_and_filters() {
    let (_dir, config_path) = temp_config_with_ws(9);
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--config",
        config_path.to_str().unwrap(),
        "--api-key",
        "sk_test",
        "poll",
        "start",
        "--json",
        "--filter",
        "message.created",
        "--filter",
        "event.notification",
        "--max-backoff",
        "30",
    ]);

    cmd.assert().success();
}

#[test]
fn poll_start_parses_without_options() {
    let (_dir, config_path) = temp_config_with_ws(9);
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--config",
        config_path.to_str().unwrap(),
        "--api-key",
        "sk_test",
        "poll",
        "start",
    ]);

    cmd.assert().success();
}
