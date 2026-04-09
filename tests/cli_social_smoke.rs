use assert_cmd::Command;

#[test]
fn feed_list_parses_and_handles_unreachable_server() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--api-key",
        "sk_test",
        "--base-url",
        "http://127.0.0.1:9",
        "feed",
        "list",
        "--page",
        "2",
        "--per-page",
        "5",
        "--following",
        "--type",
        "post",
        "--q",
        "rust",
    ]);

    cmd.assert().success();
}

#[test]
fn posts_list_me_parses_without_panic() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--api-key",
        "sk_test",
        "--base-url",
        "http://127.0.0.1:9",
        "posts",
        "list",
        "--me",
        "--page",
        "1",
        "--per-page",
        "10",
        "--visibility",
        "public",
    ]);

    cmd.assert().success();
}

#[test]
fn posts_comments_create_parses_parent_id_without_panic() {
    let mut cmd = Command::cargo_bin("agentlink").unwrap();
    cmd.args([
        "--api-key",
        "sk_test",
        "--base-url",
        "http://127.0.0.1:9",
        "posts",
        "comments",
        "create",
        "550e8400-e29b-41d4-a716-446655440000",
        "谢谢，欢迎继续补充上下文",
        "--parent-id",
        "550e8400-e29b-41d4-a716-446655440001",
    ]);

    cmd.assert().success();
}
