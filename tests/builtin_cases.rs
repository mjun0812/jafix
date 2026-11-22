mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::{Sandbox, read, stdout};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    /// 指摘の行の `[<rule-id>]` のID。caseに `rule` があればそちらを使い、どちらも無ければfixtureのファイル名である
    rule: Option<String>,
    /// 全caseに当てる `[builtin]` の中身。caseに `builtin` があれば、そちらを使う
    builtin: Option<String>,
    /// 全caseに当てる `[filters]` の中身。caseに `filters` があれば、そちらを使う
    filters: Option<String>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    /// textlintと変える理由。検査には使わない
    #[allow(dead_code)]
    note: Option<String>,
    rule: Option<String>,
    builtin: Option<String>,
    filters: Option<String>,
    /// 入力 `t.md` と同じディレクトリを根にして置くファイル。キーがパス、値が中身である
    #[serde(default)]
    files: BTreeMap<String, String>,
    text: String,
    findings: Vec<String>,
    messages: Option<Vec<String>>,
    fixed: Option<String>,
}

struct FixtureCase {
    fixture: String,
    rule: String,
    case: Case,
}

#[test]
fn builtin_and_filter_fixtures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut batches: BTreeMap<String, Vec<FixtureCase>> = BTreeMap::new();
    let mut failures = Vec::new();
    for dir in ["builtin", "filters"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        paths.sort_unstable();
        assert!(!paths.is_empty(), "{dir}: no fixtures");

        for path in paths {
            let fixture: Fixture =
                toml::from_str(&read(&path)).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let name = path.file_stem().unwrap().to_str().unwrap();
            let fixture_name = format!("{dir}/{name}.toml");
            assert!(!fixture.cases.is_empty(), "{fixture_name}: no cases");
            for case in fixture.cases {
                let rule = case
                    .rule
                    .as_deref()
                    .or(fixture.rule.as_deref())
                    .unwrap_or(name)
                    .to_owned();
                let builtin = case.builtin.as_deref().or(fixture.builtin.as_deref());
                let filters = case.filters.as_deref().or(fixture.filters.as_deref());
                // 技術文書向けルールとほかのルールとの重なりを、初期設定に依存せず確かめる。
                let mut config = String::from("[builtin]\npreset-ja-technical-writing = true\n");
                if let Some(builtin) = builtin {
                    config.push_str(&format!("{builtin}\n"));
                }
                if let Some(filters) = filters {
                    config.push_str(&format!("[filters]\n{filters}\n"));
                }
                let entry = FixtureCase {
                    fixture: fixture_name.clone(),
                    rule,
                    case,
                };
                if entry.case.files.is_empty() {
                    batches.entry(config).or_default().push(entry);
                } else {
                    // 補助ファイルのパスは、別のcaseと同じディレクトリを使うと衝突し得る。
                    check_batch(&config, &[entry], &mut failures);
                }
            }
        }
    }
    for (config, cases) in batches {
        check_batch(&config, &cases, &mut failures);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn check_batch(config: &str, cases: &[FixtureCase], failures: &mut Vec<String>) {
    let sandbox = Sandbox::new();
    let paths: Vec<_> = cases
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let path = if cases.len() == 1 {
                for (name, content) in &entry.case.files {
                    sandbox.write(name, content);
                }
                "t.md".to_owned()
            } else {
                format!("{i}/t.md")
            };
            sandbox.write(&path, &format!("{}\n", entry.case.text));
            path
        })
        .collect();
    sandbox.write_xdg_config(config);

    let output = sandbox
        .command()
        .arg("check")
        .args(&paths)
        .output()
        .unwrap();
    if !matches!(output.status.code(), Some(0 | 1)) {
        failures.push(format!(
            "{}/{}: check exited {:?}: {}",
            cases[0].fixture,
            cases[0].case.name,
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
        return;
    }
    let mut by_path: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for line in stdout(&output).lines() {
        if let Some((path, rest)) = line.split_once(':') {
            by_path.entry(path).or_default().push(rest);
        }
    }

    let mut to_fix = Vec::new();
    for (path, entry) in paths.iter().zip(cases) {
        let label = format!("{}: {}", entry.fixture, entry.case.name);
        let prefix = format!("[{}] ", entry.rule);
        let (findings, messages): (Vec<_>, Vec<_>) = by_path
            .get(path.as_str())
            .into_iter()
            .flatten()
            .filter_map(|line| {
                let (position, report) = line.split_once(": ")?;
                Some((position, report.strip_prefix(&prefix)?))
            })
            .unzip();
        if findings != entry.case.findings {
            failures.push(format!(
                "{label}: findings {findings:?} != {:?}",
                entry.case.findings
            ));
        }
        if let Some(expected) = &entry.case.messages
            && &messages != expected
        {
            failures.push(format!("{label}: messages {messages:?} != {expected:?}"));
        }
        if let Some(expected) = &entry.case.fixed {
            to_fix.push((
                path,
                label,
                format!("{expected}\n"),
                expected != &entry.case.text,
            ));
        }
    }
    for pass in 1..=2 {
        if to_fix.is_empty() {
            break;
        }
        let output = sandbox
            .command()
            .arg("fix")
            .args(to_fix.iter().map(|(path, ..)| path))
            .output()
            .unwrap();
        if !output.status.success() || !output.stdout.is_empty() {
            failures.push(format!(
                "{}: fix (pass {pass}) exited {:?}, stdout: {:?}, stderr: {}",
                to_fix[0].1,
                output.status.code(),
                stdout(&output),
                String::from_utf8_lossy(&output.stderr)
            ));
            break;
        }
        to_fix.retain(|(path, label, expected, changed)| {
            let fixed = read(&sandbox.path(path));
            if &fixed != expected {
                failures.push(format!(
                    "{label}: fix (pass {pass}) {fixed:?} != {expected:?}"
                ));
                return false;
            }
            *changed
        });
    }
}
