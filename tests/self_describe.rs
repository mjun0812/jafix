mod common;

use std::collections::BTreeSet;

use common::{Sandbox, stdout};
use serde_json::Value;

const SUBCOMMANDS: [&str; 5] = ["check", "fix", "rules", "hook", "mcp"];

/// 標準出力が1行のJSONオブジェクトであることを確かめて読む
fn json_line(output: &std::process::Output) -> Value {
    let text = stdout(output);
    assert_eq!(text.lines().count(), 1, "{text}");
    let value: Value = serde_json::from_str(text).unwrap();
    assert!(value.is_object(), "{text}");
    value
}

fn usage(sandbox: &Sandbox) -> Value {
    let output = sandbox.run(&["--usage"]);
    assert_eq!(output.status.code(), Some(0));
    json_line(&output)
}

fn command<'a>(usage: &'a Value, name: &str) -> &'a Value {
    usage["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["name"] == name)
        .unwrap()
}

fn object_keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect()
}

/// `jafix rules` のテキストの出力から、`id` の行の有効・無効とpresetの欄を返す
fn rule_line<'a>(output: &'a str, id: &str) -> (&'a str, &'a str) {
    let fields: Vec<&str> = output
        .lines()
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .find(|fields| fields[0] == id)
        .unwrap();
    assert_eq!(fields.len(), 3, "{fields:?}");
    (fields[1], fields[2])
}

fn rules_text(sandbox: &Sandbox, args: &[&str]) -> String {
    let output = sandbox.run(&[&["rules"], args].concat());
    assert_eq!(output.status.code(), Some(0));
    stdout(&output).to_string()
}

#[test]
fn help_shows_examples_and_exit_codes() {
    let sandbox = Sandbox::new();
    let root = sandbox.run(&["--help"]);
    assert_eq!(root.status.code(), Some(0));
    assert!(stdout(&root).lines().any(|line| line == "例:"));
    for name in SUBCOMMANDS {
        let output = sandbox.run(&[name, "--help"]);
        assert_eq!(output.status.code(), Some(0), "{name}");
        let help = stdout(&output);
        assert!(help.lines().any(|line| line == "例:"), "{name}: {help}");
        assert!(
            help.lines().any(|line| line == "終了コード:"),
            "{name}: {help}"
        );
    }
}

#[test]
fn usage_examples_and_exit_codes_match_help() {
    let sandbox = Sandbox::new();
    let usage = usage(&sandbox);
    let root_help = sandbox.run(&["--help"]);
    for example in usage["examples"].as_array().unwrap() {
        let line = example["command"].as_str().unwrap();
        assert!(stdout(&root_help).contains(line), "{line}");
    }
    for name in SUBCOMMANDS {
        let output = sandbox.run(&[name, "--help"]);
        let help = stdout(&output);
        let command = command(&usage, name);
        let examples = command["examples"].as_array().unwrap();
        assert!(!examples.is_empty(), "{name}");
        for example in examples {
            let line = example["command"].as_str().unwrap();
            assert!(help.contains(line), "{name}: {line}");
            assert!(
                help.contains(example["description"].as_str().unwrap()),
                "{name}: {line}"
            );
        }
        for exit_code in command["exit_codes"].as_array().unwrap() {
            let meaning = exit_code["meaning"].as_str().unwrap();
            assert!(help.contains(meaning), "{name}: {meaning}");
        }
    }
}

#[test]
fn no_arguments_exits_2() {
    let output = Sandbox::new().run(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
}

#[test]
fn usage_lists_every_command() {
    let usage = usage(&Sandbox::new());
    assert_eq!(usage["schema_version"], 1);
    assert_eq!(usage["name"], "jafix");
    assert_eq!(usage["version"], env!("CARGO_PKG_VERSION"));
    let names: Vec<&str> = usage["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|command| command["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, SUBCOMMANDS);
}

#[test]
fn usage_describes_arguments() {
    let usage = usage(&Sandbox::new());
    let args = command(&usage, "check")["args"].as_array().unwrap();
    let by_long = |long: &str| args.iter().find(|arg| arg["long"] == long).unwrap();

    let output_format = by_long("--output-format");
    assert_eq!(output_format["default"], "text");
    assert_eq!(output_format["takes_value"], true);
    assert_eq!(
        output_format["possible_values"],
        serde_json::json!(["text", "json"])
    );

    let diff = by_long("--diff");
    assert_eq!(diff["takes_value"], false);
    assert_eq!(diff["possible_values"], serde_json::json!([]));

    let files = args.iter().find(|arg| arg["positional"] == true).unwrap();
    assert_eq!(files["multiple"], true);
    assert_eq!(files["long"], Value::Null);
}

#[test]
fn usage_lists_exit_codes() {
    let usage = usage(&Sandbox::new());
    let expected: [(&str, &[u64]); 5] = [
        ("check", &[0, 1, 2]),
        ("fix", &[0, 2]),
        ("rules", &[0, 2]),
        ("hook", &[0, 1, 2]),
        ("mcp", &[0, 2]),
    ];
    for (name, codes) in expected {
        let actual: Vec<u64> = command(&usage, name)["exit_codes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|exit_code| exit_code["code"].as_u64().unwrap())
            .collect();
        assert_eq!(actual, codes, "{name}");
    }
}

#[test]
fn usage_rejects_other_arguments() {
    let output = Sandbox::new().run(&["--usage", "check"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
}

#[test]
fn rules_lists_all_rules_with_defaults() {
    let sandbox = Sandbox::new();
    let output = rules_text(&sandbox, &[]);
    assert_eq!(output.lines().count(), 102);
    assert_eq!(rule_line(&output, "fullwidth-paren"), ("有効", "-"));
    assert_eq!(
        rule_line(&output, "sentence-length"),
        ("無効", "preset-ja-technical-writing")
    );
}

#[test]
fn rules_reflects_project_config() {
    let sandbox = Sandbox::new();
    sandbox.write(
        ".jafix.toml",
        "[builtin]\nsentence-length = true\nfullwidth-paren = false\n",
    );
    let output = rules_text(&sandbox, &[]);
    assert_eq!(rule_line(&output, "sentence-length").0, "有効");
    assert_eq!(rule_line(&output, "fullwidth-paren").0, "無効");
}

#[test]
fn rules_reflects_presets() {
    let sandbox = Sandbox::new();
    sandbox.write(".jafix.toml", "[builtin]\npreset-ja-spacing = true\n");
    let output = rules_text(&sandbox, &[]);
    assert_eq!(
        rule_line(&output, "ja-space-around-code"),
        ("有効", "preset-ja-spacing")
    );
    assert_eq!(
        rule_line(&output, "ja-space-around-emphasis"),
        ("無効", "preset-ja-spacing")
    );
}

#[test]
fn rules_finds_config_from_path() {
    let sandbox = Sandbox::new();
    sandbox.write("project/.jafix.toml", "[builtin]\nsentence-length = true\n");
    sandbox.write("project/docs/guide.md", "本文。\n");
    sandbox.write("elsewhere/.keep", "");

    let mut in_subdirectory = sandbox.command();
    in_subdirectory
        .current_dir(sandbox.path("project/docs"))
        .arg("rules");
    let output = in_subdirectory.output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(rule_line(stdout(&output), "sentence-length").0, "有効");

    let mut from_elsewhere = sandbox.command();
    from_elsewhere
        .current_dir(sandbox.path("elsewhere"))
        .args(["rules", "../project/docs/guide.md"]);
    let output = from_elsewhere.output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(rule_line(stdout(&output), "sentence-length").0, "有効");

    let without_config = rules_text(&sandbox, &["elsewhere"]);
    assert_eq!(rule_line(&without_config, "sentence-length").0, "無効");
}

#[test]
fn rules_json_lists_all_rules() {
    let output = Sandbox::new().run(&["rules", "--output-format", "json"]);
    assert_eq!(output.status.code(), Some(0));
    let value = json_line(&output);
    assert_eq!(value["schema_version"], 1);
    let rules = value["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 102);
    let enabled = rules.iter().filter(|rule| rule["enabled"] == true).count();
    assert_eq!(enabled, 10);
    let sentence_length = rules
        .iter()
        .find(|rule| rule["id"] == "sentence-length")
        .unwrap();
    assert_eq!(
        sentence_length["presets"],
        serde_json::json!(["preset-ja-technical-writing"])
    );
}

#[test]
fn rules_exits_2_for_invalid_config() {
    let sandbox = Sandbox::new();
    sandbox.write(".jafix.toml", "[builtin]\nunknown-rule = true\n");
    for format in ["text", "json"] {
        let output = sandbox.run(&["rules", "--output-format", format]);
        assert_eq!(output.status.code(), Some(2), "{format}");
        assert_eq!(stdout(&output), "", "{format}");
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert!(stderr.starts_with("jafix: "), "{format}: {stderr}");
    }
}

#[test]
fn rules_exits_2_for_missing_path() {
    let output = Sandbox::new().run(&["rules", "missing.md"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_schema_is_json_schema() {
    let output = Sandbox::new().run(&["check", "--schema"]);
    assert_eq!(output.status.code(), Some(0));
    let schema = json_line(&output);
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
}

#[test]
fn check_output_matches_schema_keys() {
    let sandbox = Sandbox::new();
    sandbox.write("notes.md", "日本語 Rustです。\n");
    let output = sandbox.run(&["check", "--output-format", "json", "notes.md"]);
    assert_eq!(output.status.code(), Some(1));
    let schema = json_line(&sandbox.run(&["check", "--schema"]));
    assert_eq!(
        object_keys(&json_line(&output)),
        object_keys(&schema["properties"])
    );
}

#[test]
fn rules_output_matches_schema_keys() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(&["rules", "--output-format", "json"]);
    let schema = sandbox.run(&["rules", "--schema"]);
    assert_eq!(schema.status.code(), Some(0));
    let schema = json_line(&schema);
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(
        object_keys(&json_line(&output)),
        object_keys(&schema["properties"])
    );
}

#[test]
fn schema_rejects_other_arguments() {
    let sandbox = Sandbox::new();
    for args in [
        ["check", "--schema", "notes.md"],
        ["rules", "--schema", "--output-format=json"],
    ] {
        let output = sandbox.run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert_eq!(stdout(&output), "", "{args:?}");
    }
}
