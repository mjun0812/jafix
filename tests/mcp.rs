mod common;

use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, ChildStdout, Stdio};

use common::Sandbox;
use serde_json::{Value, json};

#[test]
fn stdio_mcp_checks_and_fixes_the_supplied_text_with_project_rules() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nja-no-successive-word = false\n");
    sandbox.write(
        "project/.jafix.toml",
        "[builtin]\nja-no-successive-word = true\n",
    );
    let original = "日本語 Rustです。\n結果をを確認します。\n";
    let path = sandbox.write("project/note.md", original);
    let mut process = sandbox
        .command()
        .args(["mcp", "--project-dir", "project"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = process.stdin.take().unwrap();
    let mut output = BufReader::new(process.stdout.take().unwrap());
    let hello = request(
        &mut input,
        &mut output,
        1,
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "jafix-test", "version": "1" }
        }),
    );
    assert_eq!(hello["serverInfo"]["name"], "jafix");
    writeln!(
        input,
        "{}",
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
    )
    .unwrap();

    let tools = request(&mut input, &mut output, 2, "tools/list", json!({}));
    for name in ["check_text", "fix_text"] {
        let tool = tools["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|tool| tool["name"] == name)
            .unwrap();
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert!(tool["outputSchema"].is_object());
    }
    let checked = request(
        &mut input,
        &mut output,
        3,
        "tools/call",
        json!({
            "name": "check_text", "arguments": { "text": original }
        }),
    );
    let findings = &checked["structuredContent"]["findings"];
    assert_eq!(findings[0]["rule_id"], "space-between-ja-and-alnum");
    assert_eq!(findings[0]["fixable"], true);
    assert_eq!(findings[1]["rule_id"], "ja-no-successive-word");
    assert_eq!(findings[1]["fixable"], false);

    let fixed = request(
        &mut input,
        &mut output,
        4,
        "tools/call",
        json!({
            "name": "fix_text", "arguments": { "text": original }
        }),
    );
    assert_eq!(
        fixed["structuredContent"]["text"],
        "日本語Rustです。\n結果をを確認します。\n"
    );
    assert_eq!(fixed["structuredContent"]["findings"], json!([findings[1]]));
    assert_eq!(std::fs::read_to_string(path).unwrap(), original);
    drop(input);
    assert!(process.wait().unwrap().success());
}

fn request(
    input: &mut ChildStdin,
    output: &mut BufReader<ChildStdout>,
    id: u32,
    method: &str,
    params: Value,
) -> Value {
    writeln!(
        input,
        "{}",
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
    )
    .unwrap();
    let mut line = String::new();
    assert_ne!(
        output.read_line(&mut line).unwrap(),
        0,
        "{method}: no response"
    );
    let response: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(response["id"], id, "{response}");
    assert!(response.get("result").is_some(), "{response}");
    response["result"].clone()
}
