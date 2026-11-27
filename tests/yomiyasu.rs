mod common;

use common::{Sandbox, read, stdout};

const RULES: &str = "[builtin]\nyomiyasu-bold-not-rendered = true\nyomiyasu-instruction-residue = true\nyomiyasu-chat-response = true\n";

#[test]
fn yomiyasu_rules_are_opt_in_and_only_report() {
    let sandbox = Sandbox::new();
    let text = "選択肢は**「有効」**です。\n\nこの記事では、AIっぽさが出ないように書きます 🚀\n\nご質問ありがとうございます。\n";
    let path = sandbox.write("t.md", text);
    let off = sandbox.run(&["check", "t.md"]);
    assert_eq!(off.status.code(), Some(0), "{off:?}");
    sandbox.write_xdg_config(RULES);

    let check = sandbox.run(&["check", "t.md"]);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    assert_eq!(
        stdout(&check),
        "t.md:1:5: [yomiyasu-bold-not-rendered] 太字の記号がそのまま表示される。記号と括弧や空白の位置を確認する\n\
         t.md:3:1: [yomiyasu-instruction-residue] 執筆時の指示が本文に残っていないか確認する\n\
         t.md:5:1: [yomiyasu-chat-response] チャット応答の定型句が本文に残っていないか確認する\n"
    );
    let fix = sandbox.run(&["fix", "t.md"]);
    assert_eq!(fix.status.code(), Some(0), "{fix:?}");
    assert_eq!(read(&path), text);
}

#[test]
fn broken_bold_checks_blocks_and_skips_markers_in_code_and_escaped_text() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nyomiyasu-bold-not-rendered = true\n");
    sandbox.write(
        "t.md",
        concat!(
            "前**「複数行に\nまたがる」**後\n\n",
            "前**「名前`x`」**後\n\n",
            "| label |\n| --- |\n| 前**「設定」**後 |\n\n",
            "選択肢は **「有効」** です。\n\n",
            "インラインコードは `前**「設定」**後` です。\n\n",
            "```md\n前**「設定」**後\n```\n\n",
            "エスケープした\\*\\*「有効」\\*\\*です。\n",
        ),
    );
    let check = sandbox.run(&["check", "t.md"]);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    let findings: Vec<_> = stdout(&check)
        .lines()
        .map(|line| line.splitn(4, ':').take(3).collect::<Vec<_>>().join(":"))
        .collect();
    assert_eq!(findings, ["t.md:1:2", "t.md:4:2", "t.md:8:4"], "{check:?}");
}

#[test]
fn instruction_residue_keeps_reader_scope_and_adjacent_references() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nyomiyasu-instruction-residue = true\n");
    sandbox.write(
        "t.md",
        concat!(
            "このマニュアルでは内部実装は扱いません。\n\n",
            "この記事では運用の詳細は扱いません。詳しくは[別のガイド](guide.md)を参照してください。\n\n",
            "この記事では運用の詳細は扱いません。\n\n",
            "> この記事では、AIっぽさが出ないように書きます。\n\n",
            "引用「この記事では飾らない言葉で書きます。」の処理を試します。\n",
        ),
    );
    let check = sandbox.run(&["check", "t.md"]);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    assert_eq!(stdout(&check).lines().count(), 1, "{check:?}");
    assert!(stdout(&check).starts_with("t.md:5:1:"), "{check:?}");
}

#[test]
fn chat_response_detects_chat_formulas_without_flagging_regular_instructions() {
    let sandbox = Sandbox::new();
    sandbox.write_xdg_config("[builtin]\nyomiyasu-chat-response = true\n");
    sandbox.write(
        "t.md",
        concat!(
            "必要なら管理者に確認してください。\n\n",
            "引用例は「ご質問ありがとうございます。」です。\n\n",
            "ご希望があれば、続けて設定例を紹介します 👍\n\n",
            "必要ならこの後の手順を確認します。\n\n",
            "ご質問`x`ありがとうございます。\n\n",
            "**ご質問**ありがとうございます。\n",
        ),
    );
    let check = sandbox.run(&["check", "t.md"]);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    let mut findings = stdout(&check).lines();
    assert!(
        findings.next().unwrap().starts_with("t.md:5:1:"),
        "{check:?}"
    );
    assert!(
        findings.next().unwrap().starts_with("t.md:11:1:"),
        "{check:?}"
    );
    assert!(findings.next().is_none(), "{check:?}");
}

#[test]
fn phrase_and_placeholder_rules_are_off_until_listed_in_builtin() {
    const IDS: [&str; 6] = [
        "yomiyasu-meta-filler",
        "yomiyasu-summary-restatement",
        "yomiyasu-meta-intro",
        "yomiyasu-count-declaration",
        "yomiyasu-ai-phrase",
        "yomiyasu-placeholder-residue",
    ];
    let sandbox = Sandbox::new();
    let states = |sandbox: &Sandbox| -> Vec<String> {
        let output = sandbox.run(&["rules"]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        IDS.iter()
            .map(|id| {
                let line = stdout(&output)
                    .lines()
                    .find(|line| line.split('\t').next() == Some(id))
                    .unwrap_or_else(|| panic!("{id} is not listed"));
                line.split_once('\t').unwrap().1.to_owned()
            })
            .collect()
    };
    assert_eq!(states(&sandbox), ["無効\t-"; 6]);

    let config: String = IDS.iter().map(|id| format!("{id} = true\n")).collect();
    sandbox.write_xdg_config(&format!("[builtin]\n{config}"));
    assert_eq!(states(&sandbox), ["有効\t-"; 6]);
    sandbox.write("t.md", "重要なのは、継続です。担当は〇〇です。\n");
    let check = sandbox.run(&["check", "t.md"]);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    assert_eq!(
        stdout(&check),
        "t.md:1:1: [yomiyasu-meta-filler] 定型の前置きや結び「重要なのは」が本文に残っていないか確認する\n\
         t.md:1:15: [yomiyasu-placeholder-residue] 仮置きの文字「〇〇」が本文に残っていないか確認する\n"
    );

    sandbox.write_xdg_config("[builtin]\nyomiyasu-ai-phrase = { strict = true }\n");
    let invalid = sandbox.run(&["rules"]);
    assert_eq!(invalid.status.code(), Some(2), "{invalid:?}");
}

#[test]
fn sentence_shape_rules_are_off_until_listed_in_builtin() {
    const IDS: [&str; 6] = [
        "yomiyasu-short-mochiron",
        "yomiyasu-say-hedge",
        "yomiyasu-fragment-run",
        "yomiyasu-cleft-list-ending",
        "yomiyasu-copy-comma-noun",
        "yomiyasu-bare-verb-chain",
    ];
    let sandbox = Sandbox::new();
    let states = |sandbox: &Sandbox| -> Vec<String> {
        let output = sandbox.run(&["rules"]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        IDS.iter()
            .map(|id| {
                let line = stdout(&output)
                    .lines()
                    .find(|line| line.split('\t').next() == Some(id))
                    .unwrap_or_else(|| panic!("{id} is not listed"));
                line.split_once('\t').unwrap().1.to_owned()
            })
            .collect()
    };
    let text = "効く。重い。安い。\n\n品質が高いと言えるでしょう。\n";
    sandbox.write("t.md", text);
    assert_eq!(states(&sandbox), ["無効\t-"; 6]);
    let quiet = sandbox.run(&["check", "t.md"]);
    assert_eq!(quiet.status.code(), Some(0), "{quiet:?}");

    let config: String = IDS.iter().map(|id| format!("{id} = true\n")).collect();
    sandbox.write_xdg_config(&format!("[builtin]\n{config}"));
    assert_eq!(states(&sandbox), ["有効\t-"; 6]);
    let check = sandbox.run(&["check", "t.md"]);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    assert_eq!(
        stdout(&check),
        "t.md:1:1: [yomiyasu-fragment-run] 短い文を3つ以上続けている\n\
         t.md:3:1: [yomiyasu-say-hedge] 文末の「と言えるでしょう」を使っている\n"
    );

    sandbox.write_xdg_config("[builtin]\nyomiyasu-say-hedge = { strict = true }\n");
    let invalid = sandbox.run(&["rules"]);
    assert_eq!(invalid.status.code(), Some(2), "{invalid:?}");
}
