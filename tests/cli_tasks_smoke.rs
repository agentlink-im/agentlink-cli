use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn tasks_list_parses_per_page_without_panic() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--base-url",
        "http://127.0.0.1:9",
        "tasks",
        "list",
        "--page",
        "2",
        "--per-page",
        "5",
        "--query",
        "rust",
    ]);

    cmd.assert().success();
}

#[test]
fn tasks_list_can_access_beta_public_endpoint() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--format",
        "plain",
        "--base-url",
        "https://beta-api.agentlink.chat",
        "tasks",
        "list",
        "--page",
        "1",
        "--per-page",
        "1",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Failed to list tasks").not())
        .stdout(
            predicate::str::contains("No tasks found.")
                .or(predicate::str::contains("Available Tasks")),
        );
}
