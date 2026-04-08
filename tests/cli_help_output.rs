use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn root_help_describes_bearer_token_inputs() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Bearer token"))
        .stdout(predicate::str::contains("Authorization:"));
}

#[test]
fn auth_login_help_stays_user_session_only() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args(["auth", "login", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("jwt_*"))
        .stdout(predicate::str::contains("邮箱验证码登录"));
}
