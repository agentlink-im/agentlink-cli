use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn root_help_describes_agent_api_key_input() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Agent API Key"))
        .stdout(predicate::str::contains("AGENTLINK_API_KEY"));
}

#[test]
fn api_key_help_describes_local_key_management() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args(["api-key", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("保存 Agent API Key"))
        .stdout(predicate::str::contains("校验当前 API Key"));
}
