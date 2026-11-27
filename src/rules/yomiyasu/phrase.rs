//! 定型の前置きと結び、AIの言い回し。yomiyasu 3.4.1の `meta_filler` (チャット応答の2型を除く)、`summary_restatement`、
//! `meta_intro`、`count_declaration`、`ai_phrase` の検査

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::Document;
use crate::rules::builtin::Builtin;
use crate::rules::{Report, RuleId};
use crate::sentence::{self, HardBreak};

use super::{
    BRACKETS, blank, chat_response, clauses, end_closers, finding, masked_words, paired_ranges,
    prose_in, visible_text, with_undecorated,
};

pub(super) fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    let statements: Vec<_> = STATEMENTS
        .iter()
        .filter(|statement| builtin.is_enabled(statement.rule))
        .collect();
    if builtin.yomiyasu_meta_filler || !statements.is_empty() {
        // 見出し・表のセル・引用は見ない
        for paragraph in document.paragraphs.iter().filter(|p| !p.blockquote) {
            for sentence in sentence::split(src, paragraph, HardBreak::ListItems) {
                let text = visible_text(src, &sentence.range, document);
                if builtin.yomiyasu_meta_filler
                    && let Some(name) = meta_filler(
                        &text,
                        &end_closers(src, document, &paragraph.range, &sentence.range),
                    )
                {
                    reports.push(finding(
                        sentence.range.clone(),
                        RuleId::YomiyasuMetaFiller,
                        format!("定型の前置きや結び「{name}」が本文に残っていないか確認する"),
                    ));
                }
                for statement in &statements {
                    if clauses(&text).any(|clause| statement.pattern.is_match(clause)) {
                        reports.push(finding(
                            sentence.range.clone(),
                            statement.rule,
                            statement.message,
                        ));
                    }
                }
            }
        }
    }
    if builtin.yomiyasu_ai_phrase {
        report_ai_phrase(src, document, &mut reports);
    }
    reports
}

/// `meta_filler` の型をどの文に当てるか
#[derive(Clone, Copy)]
enum Target {
    /// `。` などで区切った文
    Clauses,
    /// 区切った文と、その文末の飾りを除いた文。文末に固定した型に使う
    Undecorated,
    /// かぎ括弧と角括弧の中を伏せて区切った文と、その文末の飾りを除いた文。引用や会話の中の誇張を外す型に使う
    Unquoted,
}

struct Filler {
    name: &'static str,
    pattern: Regex,
    target: Target,
}

/// yomiyasuの `FILLER_PATTERNS` の順に並べる。チャット応答の2型は `yomiyasu-chat-response` が見る
static FILLERS: LazyLock<[Filler; 13]> = LazyLock::new(|| {
    let filler = |name, pattern, target| Filler {
        name,
        pattern: Regex::new(pattern).unwrap(),
        target,
    };
    [
        filler(
            "重要なのは",
            r"^(?:まず|ここで)?(?:重要|大切|大事)なのは",
            Target::Clauses,
        ),
        filler(
            "結論から言うと",
            r"^結論から(?:言|い)(?:う|い|え)|^結論から申し上げ",
            Target::Clauses,
        ),
        filler("正直に言うと", r"^正直に言うと", Target::Clauses),
        filler("本音を言うと", r"^本音を言うと", Target::Clauses),
        filler("避けたいのは", r"^避けたいのは", Target::Clauses),
        filler("ポイントは、", r"^ポイントは、", Target::Clauses),
        filler(
            "見落とされがち",
            r"見落とされがち|面白いのはここ|まさにそこがポイント",
            Target::Clauses,
        ),
        filler(
            "いかがでしたでしょうか",
            r"いかがでした(?:でしょうか|か)?[？?。]?$",
            Target::Undecorated,
        ),
        filler(
            "ぜひ〜してみてください",
            r"ぜひ(?:参考にして|試して|活用して)(?:みて)?ください",
            Target::Clauses,
        ),
        filler(
            "参考になれば幸いです",
            concat!(
                r"参考に(?:なれ|な)ば幸い|お役に立てれ?ば幸い|ぜひご(?:活用|参考に)ください|",
                r"まずは小さく(?:始め|はじめ)(?:ましょう|てみましょう|てください|てみてください|てみませんか|",
                r"て(?:みて)?はいかが(?:ですか|でしょうか)|る(?:ことが(?:大切|大事|重要)|のがおすすめ)です)[。！？!?]?$"
            ),
            Target::Undecorated,
        ),
        filler(
            "〜に他なりません",
            r"に(?:他|ほか)なりません",
            Target::Clauses,
        ),
        filler(
            "それでは見ていきましょう",
            r"見ていきましょう[。！!]?$|深掘りしていきます",
            Target::Undecorated,
        ),
        filler(
            "〜と言っても過言ではありません",
            concat!(
                r"とも?(?:言|い)(?:い切|いき)?っても(?:過言|言い過ぎ|言いすぎ|いいすぎ)",
                r"(?:では(?:ありません|ない)|じゃ(?:ありません|ない))(?:です)?",
                r"(?:でしょう|だろう|かもしれ(?:ません|ない)|と(?:思|おも)(?:います|う)|ね|よ)?",
                r"(?:[（(][^）)]*[）)])?[。！？!?]?$"
            ),
            Target::Unquoted,
        ),
    ]
});

/// 文が当たる最初の型の名前。`closers` は、文末の飾りとして除く字である
fn meta_filler(text: &str, closers: &str) -> Option<&'static str> {
    let plain: Vec<&str> = clauses(text).collect();
    let undecorated = with_undecorated(plain.iter().copied(), closers);
    let masked = blank(text, &paired_ranges(text.char_indices(), &BRACKETS));
    let unquoted = with_undecorated(clauses(&masked), closers);
    FILLERS
        .iter()
        .find(|filler| match filler.target {
            Target::Clauses => plain.iter().any(|clause| filler.pattern.is_match(clause)),
            Target::Undecorated => undecorated
                .iter()
                .any(|clause| filler.pattern.is_match(clause)),
            Target::Unquoted => unquoted
                .iter()
                .any(|clause| filler.pattern.is_match(clause)),
        })
        .map(|filler| filler.name)
}

/// yomiyasuの `meta_filler` の15型 (チャット応答の2型を含む) のどれかが、文のどこかに当たる
pub(super) fn is_meta_filler(text: &str) -> bool {
    FILLERS.iter().any(|filler| filler.pattern.is_match(text)) || chat_response(text)
}

/// 文の型を1つ当てる指摘ルール
struct Statement {
    rule: RuleId,
    pattern: Regex,
    message: &'static str,
}

static STATEMENTS: LazyLock<[Statement; 3]> = LazyLock::new(|| {
    let statement = |rule, pattern, message| Statement {
        rule,
        pattern: Regex::new(pattern).unwrap(),
        message,
    };
    [
        statement(
            RuleId::YomiyasuSummaryRestatement,
            r"^(?:まとめると|総じて)、",
            "「まとめると、」「総じて、」で言い直していないか確認する",
        ),
        statement(
            RuleId::YomiyasuMetaIntro,
            r"^本記事では",
            "「本記事では」の予告が要るか確認する",
        ),
        statement(
            RuleId::YomiyasuCountDeclaration,
            r"(?:ポイント|理由|観点|コツ|方法)は(?:3|三|３)つ(?:です|あります)|以下の(?:3|三|３)つの(?:観点|ポイント)",
            "「ポイントは3つです」のような数の宣言が要るか確認する",
        ),
    ]
});

/// 言い回しを当てる要素の種類。言い回しごとに、当てる種類が違う
#[derive(Clone, Copy, PartialEq, Eq)]
enum Block {
    /// リスト項目の外の段落
    Paragraph,
    /// リスト項目の中の段落
    ListItem,
    Heading,
    TableCell,
}

const ALL_BLOCKS: &[Block] = &[
    Block::Paragraph,
    Block::ListItem,
    Block::Heading,
    Block::TableCell,
];

struct AiPhrase {
    name: &'static str,
    pattern: regress::Regex,
    blocks: &'static [Block],
    /// 一致した範囲と文から、言い回しに当てない文かを決める
    excludes: fn(&str, Range<usize>) -> bool,
}

static AI_PHRASES: LazyLock<[AiPhrase; 9]> = LazyLock::new(|| {
    let phrase = |name, pattern: &str, blocks, excludes| AiPhrase {
        name,
        pattern: regress::Regex::new(pattern).unwrap(),
        blocks,
        excludes,
    };
    let hidden = r"(?:リスク|問題|課題|罠|わな|落とし穴|危険性?|本質|闇|バグ)";
    let hidden_before = r"(?:各種の|様々な|さまざまな|多くの|数々の|大きな|思わぬ|見えない|本当の|深刻な|重大な|新たな)?";
    let never = |_: &str, _: Range<usize>| false;
    [
        phrase(
            "見立てです",
            r"(?:という|との|の)見立て(?:です|でした|だ|だった|である)[。．！!]*$",
            ALL_BLOCKS,
            judged_by_someone,
        ),
        phrase(
            "潜む",
            &format!(
                r"{hidden}(?:が|も|は)[、，,]?潜(?:む|ん[でだ]|み|ま|め)|に潜(?:む|んでいる|んでいた|んだ){hidden_before}{hidden}(?!な)"
            ),
            ALL_BLOCKS,
            never,
        ),
        phrase(
            "が招く",
            r"が招く(?=[一-龥々ァ-ヴー])",
            ALL_BLOCKS,
            invited_by_someone,
        ),
        phrase(
            "欠かせない存在",
            r"今や[^。]*?欠かせない存在|欠かせない存在と(?:なって|なりつつ)",
            ALL_BLOCKS,
            never,
        ),
        phrase(
            "という挙動を排除",
            r"という挙動を(?:排除|防止|阻止)",
            ALL_BLOCKS,
            never,
        ),
        phrase(
            "なんよ",
            r"なんよね?[。．！!？?]*$",
            &[Block::Paragraph, Block::ListItem],
            never,
        ),
        phrase(
            "て形の体言止め",
            r"(?:[ぁ-ん]て|[んい]で|[一-龥々]て)[、，,][^。、，,]{1,20}?を[一-龥々]{2,}[。．]$",
            &[Block::Paragraph],
            te_is_not_verb,
        ),
        phrase(
            "多岐にわたる",
            r"多岐に(?:わた|渡)(?:る|り|っ)",
            &[Block::Paragraph],
            never,
        ),
        phrase(
            "が求められている",
            r"が求められて(?:いる|います|いた|いました|おり)",
            &[Block::Paragraph],
            never,
        ),
    ]
});

/// 判断した人や組織の語。語のあとに「たち」「ら」「ども」「陣」「側」が付いてもよい
const PERSON_WORDS: &str = concat!(
    r"人|者|員|士|医|さん|様|さま|氏|先生|医師|専門家|アナリスト|エコノミスト|記者|筆者|編集部|当社|弊社|同社|会社",
    r"|政府|当局|警察|検察|日銀|銀行|気象台|省|庁|局|都|府|県|市|区|町|村|国|自治体|委員会|協会|機関|大学|研究所",
    r"|チーム|部|課|業界|社長|会長|部長|課長|店長|院長|監督|代表|教授|首相|大臣|知事|市長|私|僕|わたし|ぼく|我々",
    r"|われわれ|彼|彼女|あなた|皆さん"
);

/// 「見立て」より前に、判断した人や組織 (「私の」「編集部の」) がある
fn judged_by_someone(sentence: &str, hit: Range<usize>) -> bool {
    static JUDGE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(&format!(
            r"(?:{PERSON_WORDS})(?:たち|ら|ども|陣|側)?(?:は|が|も|の|では|によると|によれば|としては)"
        ))
        .unwrap()
    });
    let end = hit.start + sentence[hit.start..].find("見立て").unwrap();
    JUDGE.is_match(&sentence[..end])
}

/// 「が招く」の直前が、招く人や組織 (「社長が招く客」) である
fn invited_by_someone(sentence: &str, hit: Range<usize>) -> bool {
    static INVITER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(&format!(
            concat!(
                r"(?:{}|上司|父|母|親|夫|妻|兄|姉|弟|妹|祖父|祖母|息子|娘|子ども|子供|家族|友人",
                r"|知人|友達|先輩|後輩|同僚|店|当店|学校|団体|館|団|主催者|運営)(?:たち|ら|ども|陣|側)?$"
            ),
            PERSON_WORDS
        ))
        .unwrap()
    });
    INVITER.is_match(&sentence[..hit.start])
}

/// 「て」「で」が動詞のて形でない (「について、」「に応じて、」「にて、」)
fn te_is_not_verb(sentence: &str, hit: Range<usize>) -> bool {
    const NOT_VERB: &[&str] = &[
        "について",
        "によって",
        "として",
        "に関して",
        "に対して",
        "を通して",
        "にとって",
        "につれて",
        "において",
        "にあたって",
        "に当たって",
        "に際して",
        "をめぐって",
        "を巡って",
        "に沿って",
        "にわたって",
        "に先立って",
        "そして",
        "さて",
        "すべて",
        "全て",
        "かつて",
        "やがて",
        "改めて",
        "あらためて",
        "初めて",
        "はじめて",
        "決して",
        "けっして",
        "大して",
        "たいして",
        "極めて",
        "きわめて",
        "あえて",
        "敢えて",
        "かえって",
        "却って",
        "至って",
        "総じて",
        "概して",
        "がいして",
        "おしなべて",
        "強いて",
        "果たして",
        "はたして",
        "従って",
        "したがって",
        "よって",
        "次いで",
        "ついで",
        "併せて",
        "あわせて",
        "合わせて",
        "続いて",
        "揃って",
        "そろって",
        "あて",
        "宛て",
        "めがけて",
        "目がけて",
        "を通じて",
        "に応じて",
        "に基づいて",
        "に基いて",
        "に加えて",
        "に向けて",
        "を受けて",
        "をもって",
        "に従って",
        "を含めて",
        "に限って",
        "にかけて",
        "に比べて",
        "に伴って",
        "を除いて",
        "を経て",
        "にて",
        "せいで",
        "くらいで",
        "ぐらいで",
    ];
    // 一致の初めの2字が「〜て」「〜で」である
    let te_end = sentence[hit.start..]
        .char_indices()
        .nth(2)
        .map_or(sentence.len(), |(i, _)| hit.start + i);
    NOT_VERB
        .iter()
        .any(|word| sentence[..te_end].ends_with(word))
}

/// 段落・リスト・見出しの文と表のセルに、言い回しを当てる。1文に1件で、引用の中は見ない
fn report_ai_phrase(src: &str, document: &Document, reports: &mut Vec<Report>) {
    let paragraphs = document
        .paragraphs
        .iter()
        .map(|paragraph| {
            let block = if paragraph.list_item_index.is_some() {
                Block::ListItem
            } else {
                Block::Paragraph
            };
            (paragraph, block)
        })
        .chain(
            document
                .headings
                .iter()
                .map(|heading| (heading, Block::Heading)),
        )
        .filter(|(paragraph, _)| !paragraph.blockquote);
    for (paragraph, block) in paragraphs {
        for sentence in sentence::split(src, paragraph, HardBreak::ListItems) {
            report_ai_phrase_in(src, document, sentence.range, block, reports);
        }
    }
    for cell in &document.table_cells {
        let prose = prose_in(document, cell);
        let (Some(first), Some(last)) = (prose.first(), prose.last()) else {
            continue;
        };
        if !first.context.blockquote {
            report_ai_phrase_in(
                src,
                document,
                first.range.start.max(cell.start)..last.range.end.min(cell.end),
                Block::TableCell,
                reports,
            );
        }
    }
}

fn report_ai_phrase_in(
    src: &str,
    document: &Document,
    range: Range<usize>,
    block: Block,
    reports: &mut Vec<Report>,
) {
    let text = masked_words(&visible_text(src, &range, document));
    let hit = word_clauses(&text).find_map(|clause| {
        AI_PHRASES.iter().find(|phrase| {
            phrase.blocks.contains(&block)
                && phrase
                    .pattern
                    .find(clause)
                    .is_some_and(|hit| !(phrase.excludes)(clause, hit.range()))
        })
    });
    if let Some(phrase) = hit {
        reports.push(finding(
            range,
            RuleId::YomiyasuAiPhrase,
            format!("AIの文章によく出る言い回し「{}」を使っている", phrase.name),
        ));
    }
}

/// 語の規則の文の単位。`。` `．` `！` `？` `!` `?` の並びで終わり、表の区切りの `|` `｜` でも区切る
fn word_clauses(text: &str) -> impl Iterator<Item = &str> {
    static CLAUSE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[^。．！？!?|｜]+[。．！？!?]*").unwrap());
    CLAUSE
        .find_iter(text)
        .map(|clause| clause.as_str().trim())
        .filter(|clause| !clause.is_empty())
}
