mod common;

use common::{Sandbox, fixture, read, stdout};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    builtin: Option<String>,
    text: String,
    findings: Vec<String>,
    messages: Option<Vec<String>>,
    fixed: Option<String>,
}

/// `tests/fixtures/builtin/<rule>.toml` の各caseを `jafix check` (と `fixed` があれば `jafix fix`) にかけ、
/// 食い違ったcaseを名前と一緒にまとめて報告する
fn assert_cases(rule: &str) {
    let fixture: Fixture = toml::from_str(&fixture(&format!("builtin/{rule}.toml"))).unwrap();
    let suffix = format!(" [{rule}]");
    let mut failures = Vec::new();
    for case in fixture.cases {
        let sandbox = Sandbox::new();
        let path = sandbox.write("t.md", &format!("{}\n", case.text));
        if let Some(builtin) = &case.builtin {
            sandbox.write_xdg_config(&format!("[builtin]\n{builtin}\n"));
        }
        let output = sandbox.run(&["check", "t.md"]);
        // exit 2は設定の誤りか読み書きの失敗で、出力が空になり `findings = []` のcaseが素通りするので落とす
        if output.status.code() == Some(2) {
            failures.push(format!(
                "{}: check exited 2: {}",
                case.name,
                String::from_utf8_lossy(&output.stderr)
            ));
            continue;
        }
        let (findings, messages): (Vec<&str>, Vec<&str>) = stdout(&output)
            .lines()
            .filter_map(|line| line.strip_prefix("t.md:")?.strip_suffix(&suffix))
            .filter_map(|line| line.split_once(": "))
            .unzip();
        if findings != case.findings {
            failures.push(format!(
                "{}: findings {findings:?} != {:?}",
                case.name, case.findings
            ));
        }
        if let Some(expected) = &case.messages
            && &messages != expected
        {
            failures.push(format!(
                "{}: messages {messages:?} != {expected:?}",
                case.name
            ));
        }
        if let Some(expected) = &case.fixed {
            sandbox.run(&["fix", "t.md"]);
            let fixed = read(&path);
            if fixed != format!("{expected}\n") {
                failures.push(format!("{}: fixed {fixed:?} != {expected:?}", case.name));
            }
            sandbox.run(&["fix", "t.md"]);
            if read(&path) != fixed {
                failures.push(format!("{}: second fix changed the file", case.name));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn max_kanji_continuous_len() {
    assert_cases("max-kanji-continuous-len");
}

#[test]
fn arabic_kanji_numbers() {
    assert_cases("arabic-kanji-numbers");
}

#[test]
fn no_exclamation_question_mark() {
    assert_cases("no-exclamation-question-mark");
}

#[test]
fn ja_unnatural_alphabet() {
    assert_cases("ja-unnatural-alphabet");
}
