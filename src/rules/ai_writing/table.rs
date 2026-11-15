//! `@textlint-ja/preset-ai-writing` の5ルールが判定に使う語・絵文字・正規表現の表。
//! @textlint-ja/textlint-rule-preset-ai-writing 1.7.0 の `src/rules/` からscriptで作った。
//! 正規表現は `regex` crateの構文で、textlintのソースと同じ文字列である (`BOLD_LIST_ITEM` だけ先読みを外した)

/// `no-ai-list-formatting` が指摘する絵文字
pub const LIST_EMOJIS: [&str; 32] = [
    "✅", "❌", "⭐", "✨", "💯", "⚠️", "❗", "❓", "💥", "🔥", "⚡", "💪", "🚀", "💡", "🤔", "💭",
    "🧠", "🎯", "📈", "📊", "🏆", "👍", "👎", "😊", "😎", "🎉", "🌟", "📝", "📋", "✏️", "🖊️", "💼",
];

/// `no-ai-list-formatting` の太字の型。リスト項目の元の文字列の先頭に当てる。textlintの末尾の先読み `(?=\s)` は
/// `regex` crateに無いので外した。ハイフン・ダッシュの直後が空白であることは、一致の後の1文字で確かめる
pub const BOLD_LIST_ITEM: &str =
    r"^[\s]*(?:[-*+]|\d+[.)])\s+\*\*([^*]+)\*\*(?:\s*([:：])|\s+([-—–]))";

/// `no-ai-hype-expressions` の語のカテゴリ。それぞれ `disable*` オプションで止まる
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HypeCategory {
    /// 絶対性・完全性
    Absoluteness,
    /// 抽象的・感覚的
    Abstract,
    /// 権威的・予言的
    Predictive,
}

/// `no-ai-hype-expressions` の語。すべて正規表現の特殊文字を含まない文字列である
pub const HYPE_WORDS: [(HypeCategory, &str); 25] = [
    (HypeCategory::Absoluteness, "革命的な"),
    (HypeCategory::Absoluteness, "ゲームチェンジャー"),
    (HypeCategory::Absoluteness, "世界初の"),
    (HypeCategory::Absoluteness, "究極の"),
    (HypeCategory::Absoluteness, "完全に"),
    (HypeCategory::Absoluteness, "完璧な"),
    (HypeCategory::Absoluteness, "最高の"),
    (HypeCategory::Absoluteness, "最先端の"),
    (HypeCategory::Absoluteness, "大幅に"),
    (HypeCategory::Abstract, "魔法のように"),
    (HypeCategory::Abstract, "奇跡的な"),
    (HypeCategory::Abstract, "驚異的な"),
    (HypeCategory::Abstract, "可能性を解き放つ"),
    (HypeCategory::Abstract, "潜在能力を引き出す"),
    (HypeCategory::Abstract, "民主化する"),
    (HypeCategory::Abstract, "スーパーチャージ"),
    (HypeCategory::Abstract, "驚嘆させ"),
    (HypeCategory::Predictive, "業界を再定義"),
    (HypeCategory::Predictive, "未来を変える"),
    (HypeCategory::Predictive, "パラダイムシフト"),
    (HypeCategory::Predictive, "不可避の"),
    (HypeCategory::Predictive, "新たな基準を設定"),
    (HypeCategory::Predictive, "次世代の"),
    (HypeCategory::Predictive, "フロンティアを開拓"),
    (HypeCategory::Predictive, "根本的に変革"),
];

/// `no-ai-emphasis-patterns` の絵文字と太字の型で、太字の前に来る絵文字
pub const EMPHASIS_EMOJIS: [&str; 18] = [
    "ℹ️", "🔍", "✅", "❌", "⚠️", "💡", "📝", "📋", "📌", "🔗", "🎯", "🚀", "⭐", "✨", "💯", "🔥",
    "📊", "📈",
];

/// `no-ai-emphasis-patterns` の情報プレフィックスの語。すべて正規表現の特殊文字を含まない文字列である
pub const INFO_PREFIXES: [&str; 18] = [
    "注意",
    "重要",
    "ポイント",
    "メモ",
    "参考",
    "補足",
    "確認",
    "チェック",
    "推奨",
    "おすすめ",
    "検出される例",
    "推奨される表現",
    "良い例",
    "悪い例",
    "例",
    "サンプル",
    "使用例",
    "設定例",
];

/// `ai-tech-writing-guideline` のカテゴリ。textlintのオプションの名前とは、明確性が `Voice`、具体性が `Clarity` で対応する
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuidelineCategory {
    /// 簡潔性 (`disableRedundancyGuidance`)
    Redundancy,
    /// 明確性 (`disableVoiceGuidance`)
    Voice,
    /// 具体性 (`disableClarityGuidance`)
    Clarity,
    /// 一貫性 (`disableConsistencyGuidance`)
    Consistency,
    /// 構造化 (`disableStructureGuidance`)
    Structure,
}

/// `ai-tech-writing-guideline` の段落の文字列に当てるパターン
#[derive(Debug)]
pub struct Guideline {
    pub category: GuidelineCategory,
    /// `regex` crateの正規表現
    pub pattern: &'static str,
    /// 文面のうち、`<カテゴリ>: ` の後に続く部分
    pub message: &'static str,
}

/// `ai-tech-writing-guideline` のパターン。`message` は、textlintの文面の要点 (置き換えの提案など) を1行にしたものである
pub const GUIDELINES: [Guideline; 21] = [
    Guideline {
        category: GuidelineCategory::Redundancy,
        pattern: "まず最初に",
        message: "「まず最初に」は「まず」か「最初に」にできる",
    },
    Guideline {
        category: GuidelineCategory::Redundancy,
        pattern: "あらかじめ予測",
        message: "「あらかじめ予測」は「予測」にできる",
    },
    Guideline {
        category: GuidelineCategory::Redundancy,
        pattern: "することができます",
        message: "「することができます」は「できます」か「します」にできる",
    },
    Guideline {
        category: GuidelineCategory::Redundancy,
        pattern: "する必要があります",
        message: "「する必要があります」は「してください」か「します」にできる",
    },
    Guideline {
        category: GuidelineCategory::Redundancy,
        pattern: "言うまでもなく",
        message: "前置きの「言うまでもなく」は省いて、要点から書ける",
    },
    Guideline {
        category: GuidelineCategory::Voice,
        pattern: "が行われ(て|る|ます)",
        message: "受け身の「が行われる」は、「実行する」「処理する」のような具体的な動詞の能動態にできる",
    },
    Guideline {
        category: GuidelineCategory::Voice,
        pattern: "の変更を行",
        message: "「の変更を行う」は「を変更する」にできる",
    },
    Guideline {
        category: GuidelineCategory::Voice,
        pattern: "の実装を実施",
        message: "「の実装を実施する」は「を実装する」にできる",
    },
    Guideline {
        category: GuidelineCategory::Voice,
        pattern: "によって[実行処理実施]され",
        message: "受け身の「によって実行される」は、「AがBを実行する」のような能動態にできる",
    },
    Guideline {
        category: GuidelineCategory::Voice,
        pattern: "がシステムによって実行される",
        message: "受け身の「がシステムによって実行される」は、「システムがAを実行する」のような能動態にできる",
    },
    Guideline {
        category: GuidelineCategory::Voice,
        pattern: "によって実行され",
        message: "受け身の「によって実行され」は、「システムがAを実行する」のような能動態にできる",
    },
    Guideline {
        category: GuidelineCategory::Clarity,
        pattern: "高速な(?:パフォーマンス|処理|動作)",
        message: "「高速な」は、「50ms未満の応答時間」のような数値の基準にできる",
    },
    Guideline {
        category: GuidelineCategory::Clarity,
        pattern: "大幅に(?:向上|改善|削減)",
        message: "「大幅に」は、変化の数値か割合にできる",
    },
    Guideline {
        category: GuidelineCategory::Clarity,
        pattern: "効率的な",
        message: "「効率的な」は、何に対してどう効率的かを具体的に書ける",
    },
    Guideline {
        category: GuidelineCategory::Clarity,
        pattern: "適切な",
        message: "「適切な」は、何を基準に適切かの条件を具体的に書ける",
    },
    Guideline {
        category: GuidelineCategory::Clarity,
        pattern: "必要に応じて",
        message: "「必要に応じて」は、どの状況で必要かの判断基準を具体的に書ける",
    },
    Guideline {
        category: GuidelineCategory::Consistency,
        pattern: "(ユーザー.*?(?:クライアント|顧客))|(?:(?:クライアント|顧客).*?ユーザー)",
        message: "同じ対象を指す「ユーザー」と「クライアント」か「顧客」が混ざっている",
    },
    Guideline {
        category: GuidelineCategory::Consistency,
        pattern: "(設定画面.*?(?:設定ページ|環境設定))|(?:(?:設定ページ|環境設定).*?設定画面)",
        message: "同じ機能を指す「設定画面」と「設定ページ」か「環境設定」が混ざっている",
    },
    Guideline {
        category: GuidelineCategory::Consistency,
        pattern: "(です。.*?である。)|(である。.*?です。)",
        message: "「です・ます」調と「だ・である」調が混ざっている",
    },
    Guideline {
        category: GuidelineCategory::Structure,
        pattern: "また、.*?また、",
        message: "「また、」が続くので、箇条書きか段落分けで整理できる",
    },
    Guideline {
        category: GuidelineCategory::Structure,
        pattern: "(?:第一に|まず).*?(?:第二に|次に).*?(?:第三に|最後に)",
        message: "順に述べる手順は、番号付きリストか見出しで整理できる",
    },
];

/// `ai-tech-writing-guideline` の構造化で、リストの直前の段落の文字列の末尾に当てる
pub const LIST_INTRO: &str = r"(?:例えば|具体的には|詳細には|以下|次に|また)。[\s]*$";
