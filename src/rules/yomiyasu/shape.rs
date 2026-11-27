//! 文の形。yomiyasu 3.4.1の `short_mochiron`、`say_hedge`、`fragment_run`、`cleft_list_ending`、`copy_comma_noun`、
//! `bare_verb_chain` の検査

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::prose::{Document, Paragraph};
use crate::rules::builtin::Builtin;
use crate::rules::{Report, RuleId};
use crate::sentence;

use super::phrase::is_meta_filler;
use super::{
    BRACKETS, blank, clauses, end_closers, end_decoration, finding, paired_ranges, unlinked_text,
    visible_text, without_end_decoration,
};

pub(super) fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    let shapes: Vec<_> = SHAPES
        .iter()
        .filter(|shape| builtin.is_enabled(shape.rule))
        .collect();
    if !shapes.is_empty() {
        // 見出しも見る。表のセルと引用は見ない
        let blocks = document
            .paragraphs
            .iter()
            .chain(&document.headings)
            .filter(|block| !block.blockquote);
        for block in blocks {
            for sentence in sentence::list_sentences(src, block) {
                // yomiyasuは角括弧の中を伏せるので、リンクの文字列も伏せる
                let text = unquoted(Cow::Owned(unlinked_text(src, &sentence.range, document)));
                for shape in &shapes {
                    if shape_sentences(&text).any(shape.matches) {
                        reports.push(finding(sentence.range.clone(), shape.rule, shape.message));
                    }
                }
            }
        }
    }
    let mochiron = builtin.is_enabled(RuleId::YomiyasuShortMochiron);
    let cleft = builtin.is_enabled(RuleId::YomiyasuCleftListEnding);
    let fragment_run = builtin.is_enabled(RuleId::YomiyasuFragmentRun);
    if !mochiron && !cleft && !fragment_run {
        return reports;
    }
    // 見出し・表のセル・引用は見ない
    for paragraph in document.paragraphs.iter().filter(|p| !p.blockquote) {
        let sentences: Vec<_> = sentence::list_sentences(src, paragraph)
            .iter()
            .map(|sentence| {
                let text = visible_text(src, &sentence.range, document);
                (sentence.range.clone(), text)
            })
            .collect();
        if fragment_run {
            report_fragment_run(src, &sentences, &mut reports);
        }
        let mut before = String::new();
        for (index, (range, text)) in sentences.iter().enumerate() {
            let previous = |offset: usize, thematic_break: ThematicBreak| {
                previous_sentence(&format!("{before}{}", &text[..offset]), || {
                    previous_block_text(src, document, paragraph, thematic_break)
                })
            };
            let closers = || end_closers(src, document, &paragraph.range, range);
            if mochiron
                && (text.contains("もちろん、") || text.contains("勿論、"))
                && short_mochiron(text, &closers(), |offset| {
                    previous(offset, ThematicBreak::Stops)
                })
            {
                reports.push(finding(
                    range.clone(),
                    RuleId::YomiyasuShortMochiron,
                    "文頭の「もちろん、」が要るか確認する",
                ));
            }
            if cleft
                && text.contains("のは、")
                && cleft_list_ending(text, &closers(), index + 1 == sentences.len(), |offset| {
                    previous(offset, ThematicBreak::PassesThrough)
                })
            {
                reports.push(finding(
                    range.clone(),
                    RuleId::YomiyasuCleftListEnding,
                    "「〜のは、」の後を名詞の並びで終えている",
                ));
            }
            before.push_str(text);
        }
    }
    reports
}

/// 文の形を1つ当てる指摘ルール
struct Shape {
    rule: RuleId,
    matches: fn(&str) -> bool,
    message: &'static str,
}

const SHAPES: [Shape; 3] = [
    Shape {
        rule: RuleId::YomiyasuSayHedge,
        matches: say_hedge,
        message: "文末の「と言えるでしょう」を使っている",
    },
    Shape {
        rule: RuleId::YomiyasuCopyCommaNoun,
        matches: copy_comma_noun,
        message: "修飾語と短い名詞の間に読点を打って、名詞で終えている",
    },
    Shape {
        rule: RuleId::YomiyasuBareVerbChain,
        matches: bare_verb_chain,
        message: "主語も目的語もない動詞を読点で並べている",
    },
];

/// 形の規則が見る文字列。かぎ括弧・角括弧・引用符の中を、1文字ずつ半角空白にする
fn unquoted(text: Cow<'_, str>) -> Cow<'_, str> {
    static CURLY: LazyLock<Regex> = LazyLock::new(|| Regex::new("“[^“”]*”|\"[^\"]*\"").unwrap());
    let brackets = paired_ranges(text.char_indices(), &BRACKETS);
    let text = blank(text, &brackets);
    let quotes: Vec<_> = CURLY.find_iter(&text).map(|hit| hit.range()).collect();
    blank(text, &quotes)
}

/// 形の規則の文の単位。yomiyasuは行ごとに見るので、改行でも区切る
fn shape_sentences(text: &str) -> impl Iterator<Item = &str> {
    static SENTENCE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[^。！？!?\n]+[。！？!?]*").unwrap());
    SENTENCE
        .find_iter(text)
        .map(|sentence| sentence.as_str().trim())
        .filter(|sentence| !sentence.is_empty())
}

/// 文末の「と言えるでしょう」。体言の後 (「契約と言えるでしょう」) と「〜に当たると言えるでしょう」は、ある区分に
/// 当たるかの判断なので外す。形容動詞の語幹の後 (「有効と言えるでしょう」) は評価のぼかしとして当てる
fn say_hedge(sentence: &str) -> bool {
    static SAY_HEDGE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(.{0,4})と(?:言|い)えるでしょう[。！!]?$").unwrap());
    static JUDGE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:に|へ)(?:当た|あた)る(?:と(?:言|い)えるでしょう)|に該当する(?:と(?:言|い)えるでしょう)")
            .unwrap()
    });
    static NOUN_END: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[一-龯々〆ヵヶァ-ヴーｦ-ﾟA-Za-z0-9０-９Ａ-Ｚａ-ｚ]$").unwrap());
    static NA_ADJECTIVE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(concat!(
            r"(?:的|重要|有効|必要|適切|十分|妥当|有用|大切|確実|可能|重大|有益|顕著|明確|",
            r"不可欠|理想|最適|容易|困難|安全|安心)$"
        ))
        .unwrap()
    });
    let Some(hedge) = SAY_HEDGE.captures(sentence) else {
        return false;
    };
    let head = &hedge[1];
    !JUDGE.is_match(sentence) && (!NOUN_END.is_match(head) || NA_ADJECTIVE.is_match(head))
}

/// 20字以下の文で、「な」「い」「た」「の」で終わる修飾語の後に読点を打ち、6字以下の名詞で終える (「軽い、美しい仕様。」)
fn copy_comma_noun(sentence: &str) -> bool {
    static COPY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:^|、)([^、]+?[ないたの])、([一-龯々〆ヵヶぁ-ゖァ-ヴーA-Za-z0-9０-９Ａ-Ｚａ-ｚ]{1,7})。$")
            .unwrap()
    });
    // 読点の前に助詞があれば、修飾語ではなく節である (「条件を残した、やや長い出力。」)
    static MODIFIER_PARTICLE: LazyLock<regress::Regex> =
        LazyLock::new(|| regress::Regex::new(r"を|[はがで](?![ぁ-ゖ])").unwrap());
    // 述語や助詞で終わる語と、「の」を含む名詞句は、名詞で止める形ではない (「迷った、決めた。」「ある、です。」)
    static NOT_NOUN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:[るたすだてでねよわかぞぜさうくぐつぬぶむずじ]|ない|ません|しい|たい)$|[はがにでの]")
            .unwrap()
    });
    // 感動詞、接続詞、指示語 (「はい、了解。」「その、件名。」)
    const NOT_MODIFIER: [&str; 20] = [
        "はい",
        "いいえ",
        "いい",
        "また",
        "まあ",
        "あの",
        "その",
        "この",
        "どの",
        "あのね",
        "ええ",
        "いや",
        "おい",
        "なお",
        "ただ",
        "しかし",
        "ちなみ",
        "つまり",
        "さて",
        "では",
    ];
    // 「い」で終わる名詞。ほかの「い」で終わる語は形容詞とみる (「暑い、暑い。」)
    const I_NOUNS: [&str; 17] = [
        "思い",
        "想い",
        "願い",
        "違い",
        "匂い",
        "出会い",
        "味わい",
        "祝い",
        "誓い",
        "迷い",
        "笑い",
        "住まい",
        "佇まい",
        "問い",
        "扱い",
        "装い",
        "幸い",
    ];
    if sentence.chars().count() > 20 || !sentence.ends_with('。') {
        return false;
    }
    let Some(copy) = COPY.captures(sentence) else {
        return false;
    };
    let modifier = copy[1].trim();
    let noun = copy[2].strip_suffix('を').unwrap_or(&copy[2]);
    !noun.is_empty()
        && noun.chars().count() <= 6
        && modifier.chars().count() >= 2
        && !NOT_MODIFIER.contains(&modifier)
        && MODIFIER_PARTICLE.find(modifier).is_none()
        && (!noun.ends_with('い') || I_NOUNS.contains(&noun))
        && !NOT_NOUN.is_match(noun)
}

/// 30字以下の文を読点で3つ以上に分け、どれも助詞の無い動詞の終止形で終える (「走る、跳ぶ、投げる。」)
fn bare_verb_chain(sentence: &str) -> bool {
    // 「を」はいつも助詞とみる。「は」「が」「に」は、後ろが平仮名でないときに助詞とみる
    static PARTICLE: LazyLock<regress::Regex> =
        LazyLock::new(|| regress::Regex::new(r"を|[はがに](?![ぁ-ゖ])").unwrap());
    static PART: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[一-龯々ぁ-ゖァ-ヴー]*[うくぐすつぬぶむる]か?$").unwrap());
    static LAST: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[一-龯々ぁ-ゖァ-ヴー]*[うくぐすつぬぶむる]$").unwrap());
    static POLITE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?:ます|です|ません)か?$").unwrap());
    // う段の仮名で終わる、動詞でない語 (「一つ、二つ、三つ。」の数、「楽しく」の形容詞)
    static NOT_VERB: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:^[一二三四五六七八九十百千0-9０-９いくひとふたみよ]+つ|しく)か?$").unwrap()
    });
    if sentence.chars().count() > 30 || PARTICLE.find(sentence).is_some() {
        return false;
    }
    let Some(body) = sentence.strip_suffix('。') else {
        return false;
    };
    let parts: Vec<&str> = body.split('、').map(str::trim).collect();
    let Some((last, verbs)) = parts.split_last() else {
        return false;
    };
    let verb = |part: &str, pattern: &Regex| {
        pattern.is_match(part) && !POLITE.is_match(part) && !NOT_VERB.is_match(part)
    };
    parts.len() >= 3 && verbs.iter().all(|part| verb(part, &PART)) && verb(last, &LAST)
}

/// 1〜4字の短い文を3つ以上続けた並び (「効く。重い。安い。」) ごとに、並びの文の範囲を指摘する。かぎ括弧と角括弧の中は
/// 中身を除いて、短い文に数えない (「1位は「田中。山田。佐藤。」でした。」)
fn report_fragment_run(src: &str, sentences: &[(Range<usize>, String)], reports: &mut Vec<Report>) {
    static QUOTED: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"「[^「」]*」|『[^『』]*』|\[[^\[\]]*\]").unwrap());
    // yomiyasuは行ごとに見るので、行の初めからも並びを数える
    static FRAGMENT_RUN: LazyLock<regress::Regex> = LazyLock::new(|| {
        regress::Regex::with_flags(
            r"(?:^|(?<=[。！？]))(?:[^。！？、\s「」『』]{1,4}[。！]){3,}",
            "m",
        )
        .unwrap()
    });
    let mut text = String::new();
    let mut starts = Vec::new();
    let mut previous_end = None;
    for (range, sentence) in sentences {
        if let Some(end) = previous_end {
            let gap = &src[end..range.start];
            if gap.contains('\n') {
                text.push('\n');
            } else if !gap.is_empty() {
                text.push(' ');
            }
        }
        starts.push(text.len());
        text.push_str(&QUOTED.replace_all(sentence, "「」"));
        previous_end = Some(range.end);
    }
    for run in FRAGMENT_RUN.find_iter(&text) {
        let first = starts.partition_point(|&start| start <= run.start()) - 1;
        let last = starts.partition_point(|&start| start < run.end()) - 1;
        reports.push(finding(
            sentences[first].0.start..sentences[last].0.end,
            RuleId::YomiyasuFragmentRun,
            "短い文を3つ以上続けている",
        ));
    }
}

/// 文頭の「もちろん、」の後に10字以内の語だけが続く文。問いや依頼への答えの定型 (「もちろん、大丈夫です。」) と、
/// はい・いいえで答える問いの直後の文は外す
fn short_mochiron(text: &str, closers: &str, previous: impl Fn(usize) -> String) -> bool {
    static SHORT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?:もちろん|勿論)、[^。！？!?、「」『』]{1,10}[。！!]$").unwrap()
    });
    static ANSWER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?:もちろん|勿論)、(?:大丈夫|構いません|かまいません|いいですよ|いいよ|できます|可能です|です|だ)")
            .unwrap()
    });
    clauses(text).any(|clause| {
        (SHORT.is_match(clause)
            || without_end_decoration(clause, closers).is_some_and(|core| SHORT.is_match(&core)))
            && !ANSWER.is_match(clause)
            && !ends_with_question(&previous(offset_in(text, clause)))
    })
}

/// 「〜のは、」の後に名詞を並べ、述語を書かずに終える文で、前提が直前の文に無いもの。かぎ括弧と角括弧の中は伏せて
/// 読む。頭が `meta_filler` の型の文 (「重要なのは、」) は外す。句点で終わらない文は、行で折り返した文の途中でありうる
/// ので、段落の最後の文のときだけ当てる
fn cleft_list_ending(
    text: &str,
    closers: &str,
    last_sentence: bool,
    previous: impl Fn(usize) -> String,
) -> bool {
    let masked = blank(
        Cow::Borrowed(text),
        &paired_ranges(text.char_indices(), &BRACKETS),
    );
    let clauses: Vec<&str> = clauses(&masked).collect();
    clauses.iter().enumerate().any(|(index, &clause)| {
        if !(last_sentence && index + 1 == clauses.len()) && !clause.ends_with(['。', '！', '!'])
        {
            return false;
        }
        let core = without_end_decoration(clause, closers);
        let Some(head) = cleft_head(clause).or_else(|| core.as_deref().and_then(cleft_head)) else {
            return false;
        };
        if is_meta_filler(clause) || core.as_deref().is_some_and(is_meta_filler) {
            return false;
        }
        // 前提には、伏せる前の文字列を読む (かぎ括弧の中の語も前提に数える)
        let chars = masked[..offset_in(&masked, clause)].chars().count();
        let offset = text
            .char_indices()
            .nth(chars)
            .map_or(text.len(), |(i, _)| i);
        !cleft_premise(&head, &previous(offset))
    })
}

/// 「〜のは、A、B、C。」の形なら、「のは」の前までを返す
fn cleft_head(sentence: &str) -> Option<String> {
    const HEAD: &str =
        r"^(?<head>[^。！？!?]{0,60}?(?:[るたいうくすつぬむぶぐ]|な|[んい]だ))のは、";
    const NOUN_END: &str = r"(?:[一-龥々〆ヵヶァ-ヴーｦ-ﾟA-Za-z0-9０-９Ａ-Ｚａ-ｚ%％さ]|(?<!の)み)";
    const TAIL: &str = r"(?:など|の[0-9０-９一二三四五六七八九十]+[つ点])?[。！!]?$";
    // 読点だけで区切る項目。動詞から作った名詞 (言い回し、流れ、見直し) で終わる項目も認める
    static COMMA: LazyLock<regress::Regex> = LazyLock::new(|| {
        let item = format!(
            r"(?:(?!から|より)[^、，,・。！？!?「」『』（）()\[\]\sをがはにでへ]){{0,14}}(?:{NOUN_END}|もの|こと|[しきれえげぎちせ]|(?<!ばか)り)"
        );
        regress::Regex::new(&format!(r"{HEAD}{item}(?:[、，・]{item}){{2,}}{TAIL}")).unwrap()
    });
    // 「と・や・および」でも区切る項目。区切りの語は項目に含めない
    static MIXED: LazyLock<regress::Regex> = LazyLock::new(|| {
        let item = format!(
            r"(?:(?!から|より|および)[^、，,・。！？!?「」『』（）()\[\]\sをがはにでへとや]){{0,14}}(?:{NOUN_END}|もの)"
        );
        regress::Regex::new(&format!(
            r"{HEAD}(?<items>{item}(?:(?:[、，・]|と|や|および){item})+){TAIL}"
        ))
        .unwrap()
    });
    static JOINER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[とや]|および").unwrap());
    // コードなどを伏せた2字以上の空白は、名詞の仮の字にする
    static MASKED_RUN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}").unwrap());
    if !sentence.contains("のは、") {
        return None;
    }
    let sentence: String = MASKED_RUN
        .replace_all(sentence, "Ｘ")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let group = |hit: &regress::Match, name| sentence[hit.named_group(name).unwrap()].to_owned();
    if let Some(hit) = COMMA.find(&sentence) {
        return Some(group(&hit, "head"));
    }
    let hit = MIXED.find(&sentence)?;
    let items = &sentence[hit.named_group("items").unwrap()];
    let joiners: Vec<_> = JOINER.find_iter(items).collect();
    // 読点だけの2項目 (「目指したのは、速さ、軽さ。」) と、「と・や」で1回つないで最後の項目に「の」がある1つの名詞句
    // (「父と母の笑顔」) は外す
    let noun_phrase = joiners.len() == 1
        && !items.contains(['、', '，', '・'])
        && items[joiners[0].end()..].contains('の');
    (!joiners.is_empty() && !noun_phrase).then(|| group(&hit, "head"))
}

/// 直前の文が問いか、「のは」の前の動詞の語幹の漢字 (「変動したのは」なら「変動」) を含めば、前提を書いてあるとみる
fn cleft_premise(head: &str, previous: &str) -> bool {
    static STEM: LazyLock<regress::Regex> =
        LazyLock::new(|| regress::Regex::new(r"[一-龥々〆ヵヶ]+(?=[ぁ-ん]*$)").unwrap());
    let content = &previous[..question_content_end(previous)];
    content.ends_with(['？', '?', 'か'])
        || content.ends_with("か。")
        || STEM
            .find(head)
            .is_some_and(|stem| head[stem.range()].chars().any(|c| previous.contains(c)))
}

const STOPS: [char; 5] = ['。', '！', '？', '!', '?'];

/// 段落の `before` の後に続く文の、直前の文。`before` に文の終わりが無ければ、直前の段落かリスト項目の最後の文、
/// それも無ければ直前の見出しの最後の文である
fn previous_sentence(before: &str, previous_block: impl FnOnce() -> Option<String>) -> String {
    if let Some(end) = before.rfind(STOPS) {
        let end = end + before[end..].chars().next().unwrap().len_utf8();
        return last_context_sentence(&before[..end]).to_owned();
    }
    previous_block().map_or_else(String::new, |text| last_context_sentence(&text).to_owned())
}

/// 直前の文を探すとき、区切り線を越えて前の段落を見るか。yomiyasuは、`short_mochiron` では区切り線で直前の文を
/// 切り、`cleft_list_ending` では区切り線をブロックとして持たないので越える
#[derive(Clone, Copy, PartialEq, Eq)]
enum ThematicBreak {
    Stops,
    PassesThrough,
}

/// `paragraph` の直前の段落か見出しの文字列。間に表・コードブロック・HTML・引用などを挟むときと、直前が引用の中の
/// ときは `None` である。区切り線を挟むときは `thematic_break` で決める
fn previous_block_text(
    src: &str,
    document: &Document,
    paragraph: &Paragraph,
    thematic_break: ThematicBreak,
) -> Option<String> {
    // `* * *` のような区切り線は、リストの記号の並びとして読まない
    static THEMATIC_BREAK: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[ \t]*(?:(?:-[ \t]*){3,}|(?:\*[ \t]*){3,}|(?:_[ \t]*){3,})$").unwrap()
    });
    static LIST_MARKERS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[ \t]*(?:(?:[-+*]|[0-9]{1,9}[.)])(?:[ \t]+(?:\[[ xX]\][ \t]+)?|$))*[ \t]*$")
            .unwrap()
    });
    let previous = document
        .paragraphs
        .iter()
        .chain(&document.headings)
        .filter(|block| block.range.end <= paragraph.range.start)
        .max_by_key(|block| block.range.end)?;
    // 間に挟んでよいのは、空行と、リストの記号とタスクの記号の行と、`thematic_break` が通すときの区切り線だけである
    let passes = |line: &str| {
        if THEMATIC_BREAK.is_match(line) {
            return thematic_break == ThematicBreak::PassesThrough;
        }
        LIST_MARKERS.is_match(line)
    };
    (!previous.blockquote
        && src[previous.range.end..paragraph.range.start]
            .lines()
            .all(passes))
    .then(|| visible_text(src, &previous.range, document))
}

/// `text` の最後の文。文の後ろの閉じかっこと飾りを除く
fn last_context_sentence(text: &str) -> &str {
    let content = &text[..question_content_end(text)];
    let Some((last, _)) = content.char_indices().next_back() else {
        return "";
    };
    let start = content[..last]
        .rfind(STOPS)
        .map_or(0, |i| i + content[i..].chars().next().unwrap().len_utf8());
    &content[start..]
}

/// 文の後ろの空白・閉じかっこ・強調の記号・絵文字を除いた、文の中身の終わり
fn question_content_end(sentence: &str) -> usize {
    sentence
        .trim_end_matches(|c: char| {
            c.is_whitespace() || "」』）)\"'*_".contains(c) || end_decoration(c)
        })
        .len()
}

/// はい・いいえで答える問い。疑問詞のある問い (「なぜですか」) と、「か」で終わるだけの語 (「いくつか」) は含めない
fn ends_with_question(sentence: &str) -> bool {
    const SUFFIXES: [&str; 19] = [
        "ですか",
        "ますか",
        "ませんか",
        "でしょうか",
        "だろうか",
        "のか",
        "ないか",
        "うか",
        "くか",
        "ぐか",
        "すか",
        "つか",
        "ぬか",
        "ぶか",
        "むか",
        "るか",
        "たか",
        "いいか",
        "よいか",
    ];
    static WH_QUESTION: LazyLock<regress::Regex> = LazyLock::new(|| {
        regress::Regex::new(concat!(
            r"なぜ|どうして(?!も)|どのよう(?:に|な)|いつ(?!でも|も|か(?!ら))|どこ(?!でも|も|か(?!ら))|",
            r"誰(?!でも|も|か(?!ら))|何(?!でも|も|か(?!ら))|どれ(?!でも|も|か(?!ら))|どの(?![^。！？!?、]{1,12}でも)"
        ))
        .unwrap()
    });
    let content = &sentence[..question_content_end(sentence)];
    if content.is_empty() {
        return false;
    }
    let content = if content.ends_with(['？', '?']) {
        content
    } else {
        let content = content.strip_suffix('。').unwrap_or(content);
        if content.ends_with("いくつか") || !SUFFIXES.iter().any(|suffix| content.ends_with(suffix))
        {
            return false;
        }
        content
    };
    WH_QUESTION.find(content).is_none()
}

/// `text` の部分文字列 `part` の、`text` の中での位置
fn offset_in(text: &str, part: &str) -> usize {
    part.as_ptr() as usize - text.as_ptr() as usize
}
