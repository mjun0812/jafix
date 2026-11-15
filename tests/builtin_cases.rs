mod common;

use common::{Sandbox, fixture, read, stdout};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    /// 指摘の行の `[<rule-id>]` のID。無ければfixtureのファイル名である
    rule: Option<String>,
    /// 全caseに当てる `[builtin]` の中身。caseに `builtin` があれば、そちらを使う
    builtin: Option<String>,
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
    let suffix = format!(" [{}]", fixture.rule.as_deref().unwrap_or(rule));
    let mut failures = Vec::new();
    for case in &fixture.cases {
        let sandbox = Sandbox::new();
        let path = sandbox.write("t.md", &format!("{}\n", case.text));
        if let Some(builtin) = case.builtin.as_ref().or(fixture.builtin.as_ref()) {
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

#[test]
fn no_nfd() {
    assert_cases("no-nfd");
}

#[test]
fn no_invalid_control_character() {
    assert_cases("no-invalid-control-character");
}

#[test]
fn no_zero_width_spaces() {
    assert_cases("no-zero-width-spaces");
}

#[test]
fn no_hankaku_kana() {
    assert_cases("no-hankaku-kana");
}

#[test]
fn sentence_length() {
    assert_cases("sentence-length");
}

#[test]
fn max_comma() {
    assert_cases("max-comma");
}

#[test]
fn ja_no_mixed_period() {
    assert_cases("ja-no-mixed-period");
}

#[test]
fn no_unmatched_pair() {
    assert_cases("no-unmatched-pair");
}

#[test]
fn max_ten() {
    assert_cases("max-ten");
}

#[test]
fn no_doubled_joshi() {
    assert_cases("no-doubled-joshi");
}

#[test]
fn no_doubled_conjunctive_particle_ga() {
    assert_cases("no-doubled-conjunctive-particle-ga");
}

#[test]
fn no_doubled_conjunction() {
    assert_cases("no-doubled-conjunction");
}

#[test]
fn no_mix_dearu_desumasu() {
    assert_cases("no-mix-dearu-desumasu");
}

#[test]
fn ja_no_redundant_expression() {
    assert_cases("ja-no-redundant-expression");
}

#[test]
fn no_double_negative_ja() {
    assert_cases("no-double-negative-ja");
}

#[test]
fn no_dropping_the_ra() {
    assert_cases("no-dropping-the-ra");
}

#[test]
fn ja_no_weak_phrase() {
    assert_cases("ja-no-weak-phrase");
}

#[test]
fn ja_no_successive_word() {
    assert_cases("ja-no-successive-word");
}

#[test]
fn ja_no_abusage() {
    assert_cases("ja-no-abusage");
}

#[test]
fn no_ai_words() {
    assert_cases("no-ai-words");
}

#[test]
fn no_short_topic_comma() {
    assert_cases("no-short-topic-comma");
}

#[test]
fn no_ai_list_formatting() {
    assert_cases("no-ai-list-formatting");
}

#[test]
fn no_ai_hype_expressions() {
    assert_cases("no-ai-hype-expressions");
}

#[test]
fn no_ai_emphasis_patterns() {
    assert_cases("no-ai-emphasis-patterns");
}

#[test]
fn no_ai_colon_continuation() {
    assert_cases("no-ai-colon-continuation");
}

#[test]
fn ai_tech_writing_guideline() {
    assert_cases("ai-tech-writing-guideline");
}

#[test]
fn prh() {
    assert_cases("prh");
}

#[test]
fn textlint_rule_pattern() {
    assert_cases("textlint-rule-pattern");
}

#[test]
fn ja_nakaguro_or_halfwidth_space_between_katakana() {
    assert_cases("ja-nakaguro-or-halfwidth-space-between-katakana");
}

#[test]
fn ja_no_space_around_parentheses() {
    assert_cases("ja-no-space-around-parentheses");
}

#[test]
fn ja_no_space_around_slash() {
    assert_cases("ja-no-space-around-slash");
}

#[test]
fn ja_no_space_between_full_width() {
    assert_cases("ja-no-space-between-full-width");
}

#[test]
fn ja_space_between_half_and_full_width() {
    assert_cases("ja-space-between-half-and-full-width");
}

#[test]
fn ja_space_after_exclamation() {
    assert_cases("ja-space-after-exclamation");
}

#[test]
fn ja_space_after_question() {
    assert_cases("ja-space-after-question");
}

#[test]
fn ja_space_around_code() {
    assert_cases("ja-space-around-code");
}

#[test]
fn ja_space_around_link() {
    assert_cases("ja-space-around-link");
}

#[test]
fn ja_space_around_strong() {
    assert_cases("ja-space-around-strong");
}

#[test]
fn ja_space_around_emphasis() {
    assert_cases("ja-space-around-emphasis");
}

#[test]
fn no_mixed_zenkaku_and_hankaku_alphabet() {
    assert_cases("no-mixed-zenkaku-and-hankaku-alphabet");
}

#[test]
fn no_kangxi_radicals() {
    assert_cases("no-kangxi-radicals");
}

#[test]
fn jtf_style_2_1_5() {
    assert_cases("jtf-style-2-1-5");
}

#[test]
fn jtf_style_2_1_6() {
    assert_cases("jtf-style-2-1-6");
}

#[test]
fn jtf_style_2_1_8() {
    assert_cases("jtf-style-2-1-8");
}

#[test]
fn jtf_style_2_1_9() {
    assert_cases("jtf-style-2-1-9");
}

#[test]
fn jtf_style_2_2_1() {
    assert_cases("jtf-style-2-2-1");
}

#[test]
fn jtf_style_2_2_3() {
    assert_cases("jtf-style-2-2-3");
}

#[test]
fn jtf_style_3_1_1() {
    assert_cases("jtf-style-3-1-1");
}

#[test]
fn jtf_style_3_1_2() {
    assert_cases("jtf-style-3-1-2");
}

#[test]
fn jtf_style_3_3() {
    assert_cases("jtf-style-3-3");
}

#[test]
fn jtf_style_4_2_4() {
    assert_cases("jtf-style-4-2-4");
}
