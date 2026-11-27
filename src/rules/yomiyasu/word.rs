//! 語。yomiyasu 3.4.1の `renyo_noun`、`slop_vocabulary`、`katakana_term` の検査

use std::ops::Range;
use std::sync::LazyLock;

use crate::prose::Document;
use crate::rules::builtin::Builtin;
use crate::rules::{Report, RuleId};

use super::{Visible, finding, lines, masked_words, prose_in};

pub(super) fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report> {
    let mut reports = Vec::new();
    let renyo = builtin.yomiyasu_renyo_noun;
    let katakana = builtin.yomiyasu_katakana_term;
    let slop = builtin.yomiyasu_slop_vocabulary;
    if !renyo && !katakana && !slop {
        return reports;
    }
    // 見出し・段落・リストを見る。引用は見ない
    let blocks = document
        .paragraphs
        .iter()
        .chain(&document.headings)
        .filter(|block| !block.blockquote)
        .map(|block| (&block.range, false));
    // 表のセルは、連用形名詞とカタカナ語だけが見る
    let cells = document
        .table_cells
        .iter()
        .filter(|cell| {
            prose_in(document, cell)
                .first()
                .is_some_and(|part| !part.context.blockquote)
        })
        .map(|cell| (cell, true));
    for (block, cell) in blocks.chain(cells) {
        for line in lines(src, block) {
            let visible = Visible::new(src, &line, document);
            if slop && !cell {
                report_slop(&visible, &mut reports);
            }
            if !renyo && !katakana {
                continue;
            }
            // リンクの文字列、かぎ括弧と角括弧と引用符の中、英数字の識別子を伏せて見る
            let masked = masked_words(&visible.unlinked());
            if renyo {
                report_renyo(&visible, &masked, &mut reports);
            }
            if katakana {
                for hit in KATAKANA.find_iter(&masked) {
                    let word = &masked[hit.range()];
                    reports.push(finding(
                        visible.source(&masked, hit.range()),
                        RuleId::YomiyasuKatakanaTerm,
                        format!("意味の薄くなりやすいカタカナ語「{word}」を使っている"),
                    ));
                }
            }
        }
    }
    reports
}

/// 番号や記号を付けた項目の名前 (マイルストーン1、マイルストーン B) は、何を指すかが決まっているので外す
static KATAKANA: LazyLock<regress::Regex> = LazyLock::new(|| {
    let mut words = [
        "マイルストーン",
        "シナジー",
        "ソリューション",
        "エンゲージメント",
        "プラットフォーム",
        "コミットメント",
        "アライメント",
        "ナレッジ",
        "イニシアチブ",
        "バリュー",
        "スキーム",
        "インサイト",
        "エコシステム",
        "ケイパビリティ",
        "レバレッジ",
        "オポチュニティ",
        "ステークホルダー",
        "コンセンサス",
        "ロードマップ",
        "ベストプラクティス",
        "イノベーション",
        "トランスフォーメーション",
        "エクスペリエンス",
        "カスタマージャーニー",
        "タッチポイント",
        "モメンタム",
        "コアコンピタンス",
        "プレゼンス",
        "ウェルビーイング",
        "パーパス",
    ];
    words.sort_by_key(|word| std::cmp::Reverse(word.chars().count()));
    regress::Regex::new(&format!(
        "(?:{})(?![ 　]?[0-9０-９A-Za-zＡ-Ｚａ-ｚ])",
        words.join("|")
    ))
    .unwrap()
});

/// 他動詞の連用形を1語だけ名詞にした語を「の」で受ける形 (「結果の確かめ」「案の比べ」)。名詞とくっつけた複合語
/// (資金集め)、複合動詞から来た語 (見直し)、「お」を付けた形 (お選び) は、「の」のすぐ後に語が来ないので当たらない
fn report_renyo(visible: &Visible, masked: &str, reports: &mut Vec<Report>) {
    // 語と、語の後ろがその文字列のときに外す文字列 (「生活の足しになる」「育ての親」)
    const WORDS: [(&str, &[&str]); 48] = [
        ("確かめ", &[]),
        ("直し", &[]),
        ("比べ", &[]),
        ("測り", &[]),
        ("補い", &[]),
        ("決め", &[]),
        ("選び", &[]),
        ("集め", &[]),
        ("探し", &[]),
        ("捜し", &[]),
        ("整え", &[]),
        ("揃え", &[]),
        ("そろえ", &[]),
        ("重ね", &[]),
        ("外し", &[]),
        ("戻し", &[]),
        ("削り", &[]),
        ("消し", &[]),
        ("残し", &[]),
        ("伏せ", &[]),
        ("伝え", &[]),
        ("預け", &[]),
        ("並べ", &[]),
        ("分け", &[]),
        ("混ぜ", &[]),
        ("減らし", &[]),
        ("増やし", &[]),
        ("広げ", &[]),
        ("縮め", &[]),
        ("伸ばし", &[]),
        ("延ばし", &[]),
        ("遅らせ", &[]),
        ("終え", &[]),
        ("続け", &[]),
        ("動かし", &[]),
        ("移し", &[]),
        ("貼り", &[]),
        ("挙げ", &[]),
        ("数え", &[]),
        ("改め", &[]),
        ("入れ", &[]),
        ("たしかめ", &[]),
        ("くらべ", &[]),
        ("おぎない", &[]),
        ("ととのえ", &[]),
        ("育て", &["の親", "の母", "の父"]),
        ("足し", &["に"]),
        ("試し", &["に"]),
    ];
    // 「の」の前は名詞の終わりになりうる字 (伏せた字の半角空白を含む)。語の後ろは助詞か句読点・括弧・空白・行の終わりで、
    // 動詞に続く形 (入れやすい、確かめにくさ) の「や」「に」「が」と、「探しもの」の「も」は外す
    static RENYO: LazyLock<regress::Regex> = LazyLock::new(|| {
        let mut words: Vec<&str> = WORDS.iter().map(|&(word, _)| word).collect();
        words.sort_by_key(|word| std::cmp::Reverse(word.chars().count()));
        regress::Regex::new(&format!(
            concat!(
                r"(?<=[一-龥々〆ァ-ヴーぁ-んA-Za-z0-9０-９Ａ-Ｚａ-ｚ）」』\]) ])の({})",
                r"(?=が(?!た[いかくけさそ])|を|は|に(?!く[いかくけさそ])|で|も(?!の)|と|や(?!す[いかくけさそ])|へ|の",
                r"|だけ|まで|など|から|より|こそ|さえ|しか|ばかり|くらい|ぐらい|ほど|[、。，．）」』！？!?（(・／/:：|｜\s]|$)"
            ),
            words.join("|")
        ))
        .unwrap()
    });
    for hit in RENYO.find_iter(masked) {
        let word = &masked[hit.group(1).unwrap()];
        let (_, skips) = WORDS.iter().find(|&&(w, _)| w == word).unwrap();
        if skips
            .iter()
            .any(|skip| masked[hit.end()..].starts_with(skip))
        {
            continue;
        }
        reports.push(finding(
            visible.source(masked, hit.range()),
            RuleId::YomiyasuRenyoNoun,
            format!("動詞の連用形を名詞にした「の{word}」を使っている"),
        ));
    }
}

/// AIが多用する比喩的な語。一覧の語は部分一致で当て (「体温計」の「体温」にも当たる)、別の語を巻き込む語は形で当てる。
/// かぎ括弧の中も見る
fn report_slop(visible: &Visible, reports: &mut Vec<Report>) {
    const WORDS: [&str; 30] = [
        "手触り",
        "肌感",
        "肌感覚",
        "体温",
        "温度感",
        "熱量",
        "血の通った",
        "泥臭い",
        "泥臭さ",
        "解像度",
        "腹落ち",
        "メンタルモデル",
        "本質的",
        "地に足のついた",
        "等身大",
        "営み",
        "土台",
        "羅針盤",
        "起爆剤",
        "触媒",
        "虚飾",
        "境地",
        "美学",
        "深淵",
        "冷徹",
        "禁欲的",
        "優美",
        "極致",
        "宿命",
        "正本",
    ];
    let text = &visible.text;
    let mut hits: Vec<(Range<usize>, &str)> = WORDS
        .iter()
        .flat_map(|&word| {
            text.match_indices(word)
                .map(move |(i, _)| (i..i + word.len(), word))
        })
        .collect();
    hits.extend(
        SLOP_PATTERNS.iter().flat_map(|(word, pattern)| {
            pattern.find_iter(text).map(move |hit| (hit.range(), *word))
        }),
    );
    // 重なる語 (「肌感覚」の中の「肌感」) は、長いほうだけを指摘する
    hits.sort_by_key(|(range, _)| (range.start, std::cmp::Reverse(range.end)));
    let mut end = 0;
    for (range, word) in hits {
        if range.start < end {
            continue;
        }
        end = range.end;
        reports.push(finding(
            visible.source(text, range),
            RuleId::YomiyasuSlopVocabulary,
            format!("AIが多用する語「{word}」を使っている"),
        ));
    }
}

/// 部分一致では別の語を巻き込む語と、その形
static SLOP_PATTERNS: LazyLock<[(&str, regress::Regex); 7]> = LazyLock::new(|| {
    // 機器の名前を作る前の語。ここに無い漢字の語との複合語 (増幅装置、舞台装置) は当てる
    const DEVICE_PREFIXES: [&str; 158] = [
        "安全",
        "制御",
        "記憶",
        "表示",
        "入力",
        "出力",
        "測定",
        "計測",
        "冷却",
        "加熱",
        "照明",
        "通信",
        "医療",
        "実験",
        "警報",
        "処理",
        "演算",
        "記録",
        "検出",
        "検知",
        "検査",
        "分析",
        "試験",
        "製造",
        "発電",
        "変電",
        "駆動",
        "制動",
        "操舵",
        "推進",
        "噴射",
        "点火",
        "排気",
        "換気",
        "空調",
        "暖房",
        "冷房",
        "冷凍",
        "浄化",
        "濾過",
        "給水",
        "排水",
        "消火",
        "防火",
        "防犯",
        "監視",
        "撮影",
        "投影",
        "映写",
        "音響",
        "録音",
        "受信",
        "送信",
        "中継",
        "露光",
        "洗浄",
        "乾燥",
        "粉砕",
        "搬送",
        "昇降",
        "分離",
        "蒸留",
        "電源",
        "補助",
        "予備",
        "非常",
        "救命",
        "維持",
        "透析",
        "照射",
        "診断",
        "治療",
        "計算",
        "周辺",
        "端末",
        "遮断",
        "保護",
        "防護",
        "防止",
        "固定",
        "連結",
        "緩衝",
        "減速",
        "変速",
        "蓄電",
        "充電",
        "給電",
        "印刷",
        "複写",
        "読取",
        "走査",
        "識別",
        "認証",
        "暗号",
        "圧縮",
        "自動",
        "遠隔",
        "計量",
        "包装",
        "溶接",
        "切断",
        "加工",
        "研磨",
        "塗装",
        "成形",
        "射出",
        "送風",
        "集塵",
        "加湿",
        "除湿",
        "殺菌",
        "滅菌",
        "培養",
        "撹拌",
        "混合",
        "選別",
        "航法",
        "着陸",
        "発射",
        "照準",
        "起爆",
        "時限",
        "発光",
        "警告",
        "信号",
        "制震",
        "免震",
        "防振",
        "保安",
        "調整",
        "調節",
        "変換",
        "変圧",
        "整流",
        "発振",
        "映像",
        "画像",
        "光学",
        "電子",
        "油圧",
        "空圧",
        "真空",
        "排煙",
        "消音",
        "防音",
        "供給",
        "回収",
        "循環",
        "浄水",
        "揚水",
        "貯蔵",
        "運搬",
        "牽引",
        "計装",
        "電波",
        "音波",
        "取り",
        "読取り",
    ];
    let pattern = |word, pattern: &str| (word, regress::Regex::new(pattern).unwrap());
    [
        // ナビゲート、デリゲートのような外来語は、直前のカタカナで外す (レビューゲートは当てる)
        pattern(
            "ゲート",
            concat!(
                r"(?:(?<![ァ-ヴー])|(?<=レビュー)|(?<=リリース)|(?<=チェック)|(?<=デプロイ)|(?<=マージ)|(?<=テスト))",
                r"(?<!搭乗)(?<!改札)ゲート(?!ウ[ェエ]イ|ボール|キーパー)"
            ),
        ),
        // 数学の閉包と、会計・行政の台帳は、定義どおりの用語として残す
        pattern(
            "閉包",
            r"(?<!推移)(?<!反射)(?<!対称)(?<!凸)(?<!代数)(?<!代数的)閉包(?!演算)",
        ),
        pattern(
            "台帳",
            r"(?<!資産)(?<!住民基本)(?<!会計)(?<!会計の)(?<!行政の)(?<!課税)(?<!土地)(?<!家屋)(?<!備品)(?<!登記)台帳",
        ),
        pattern(
            "〜のOS",
            r"(?:思考|意思決定|仕事|人生|組織|チーム|学習|経営)の?OS(?![A-Za-z])",
        ),
        pattern("本質を突く", "本質を突"),
        // 真理値、真理表など論理学の用語と、真理子などの名前は外す
        pattern(
            "真理",
            "真理(?!値|表|関数|条件|集合|子|恵|奈|亜|絵|江|さん|ちゃん|様)",
        ),
        // カタカナと英数字の語の後 (レーザー装置) と、機器の名前を作る語の後 (安全装置) は機器の名前として外す。機器の
        // 中で動く・組み込むものと、同じ文の後ろに操作や状態の語が続く「装置」も、文字どおりの機器として外す
        pattern(
            "装置",
            &format!(
                concat!(
                    r"(?:(?<![ァ-ヴーA-Za-z0-9０-９Ａ-Ｚａ-ｚ線])|(?<=イデオロギー))(?<!{})",
                    r"装置(?!の中で動|内で動|に組み込|を制御|の制御)",
                    r"(?![^。！？!?]{{0,20}}?(?:電源|起動|停止|配線|ケーブル|スイッチ|ボタン|ランプ|点検|修理|故障|稼働",
                    r"|作動|設置|取扱|取り扱|保守|校正|電池|電圧|電流))"
                ),
                DEVICE_PREFIXES.join("|")
            ),
        ),
    ]
});
