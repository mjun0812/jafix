//! トークンの並びと文字列の辞書。`DOUBLE_NEGATIVES` 以外は、textlintのpackageの辞書からscriptで作った

use super::pattern::Expect;
use super::pattern::Field::*;

/// 名前の付いたトークンの並び
#[derive(Debug)]
pub struct Phrase {
    pub name: &'static str,
    pub pattern: &'static [Expect],
}

/// 誤用の文字列と、置き換え後の文字列
#[derive(Debug)]
pub struct Word {
    pub pattern: &'static str,
    pub expected: &'static str,
}

/// 冗長な表現の辞書の1項目
#[derive(Debug)]
pub struct Redundant {
    pub id: &'static str,
    pub pattern: &'static [Expect],
    pub allows: &'static [&'static str],
}

const NAI: (super::pattern::Field, &[&str]) = (BasicForm, &["ない", "無い"]);
const ADJECTIVE: (super::pattern::Field, &[&str]) = (Pos, &["形容詞"]);
const PARTICLE: (super::pattern::Field, &[&str]) = (Pos, &["助詞"]);
const NOUN: (super::pattern::Field, &[&str]) = (Pos, &["名詞"]);
const VERB: (super::pattern::Field, &[&str]) = (Pos, &["動詞"]);

const fn expect(fields: &'static [(super::pattern::Field, &'static [&'static str])]) -> Expect {
    Expect {
        fields,
        skippable: false,
    }
}

/// `no-double-negative-ja` の並び。textlint-rule-no-double-negative-ja 2.0.1 の `lib/rules/` の14個のうち、
/// 最後のトークンが2通りある「ないこともない」を2つに分けた
pub const DOUBLE_NEGATIVES: [Phrase; 15] = [
    Phrase {
        name: "なくもない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Surface, &["も"]), PARTICLE]),
            expect(&[NAI, ADJECTIVE]),
        ],
    },
    Phrase {
        name: "なくはない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Surface, &["は"]), PARTICLE]),
            expect(&[NAI, ADJECTIVE]),
        ],
    },
    Phrase {
        name: "ないではない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Surface, &["で"]), (ConjugatedForm, &["連用形"])]),
            expect(&[(Surface, &["は"]), PARTICLE]),
            expect(&[NAI, ADJECTIVE]),
        ],
    },
    Phrase {
        name: "ないでもない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Surface, &["で"]), PARTICLE]),
            expect(&[(Surface, &["も"]), PARTICLE]),
            expect(&[NAI, ADJECTIVE]),
        ],
    },
    Phrase {
        name: "ないものでもない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Reading, &["モノ"]), NOUN]),
            expect(&[(ConjugatedForm, &["連用形"]), (BasicForm, &["だ"])]),
            expect(&[(Surface, &["も"]), PARTICLE]),
            expect(&[NAI, ADJECTIVE]),
        ],
    },
    Phrase {
        name: "ないものではない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Reading, &["モノ"]), NOUN]),
            expect(&[(ConjugatedForm, &["連用形"]), (BasicForm, &["だ"])]),
            expect(&[(Surface, &["は"]), PARTICLE]),
            expect(&[NAI, ADJECTIVE]),
        ],
    },
    Phrase {
        name: "ないことはない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Reading, &["コト"]), NOUN]),
            expect(&[(Surface, &["は"]), PARTICLE]),
            expect(&[NAI]),
        ],
    },
    Phrase {
        name: "ないこともない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Reading, &["コト"]), NOUN]),
            expect(&[(Surface, &["も"]), PARTICLE]),
            expect(&[NAI, (ConjugatedType, &["特殊・ナイ"])]),
        ],
    },
    Phrase {
        name: "ないこともない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Reading, &["コト"]), NOUN]),
            expect(&[(Surface, &["も"]), PARTICLE]),
            expect(&[NAI, ADJECTIVE]),
        ],
    },
    Phrase {
        name: "ないわけでもない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Reading, &["ワケ"]), NOUN]),
            expect(&[(ConjugatedForm, &["連用形"]), (BasicForm, &["だ"])]),
            expect(&[(Surface, &["も"]), PARTICLE]),
            expect(&[NAI, ADJECTIVE]),
        ],
    },
    Phrase {
        name: "ないわけではない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Reading, &["ワケ"]), NOUN]),
            expect(&[(ConjugatedForm, &["連用形"]), (BasicForm, &["だ"])]),
            expect(&[(Surface, &["は"]), PARTICLE]),
            expect(&[NAI]),
        ],
    },
    Phrase {
        name: "ないといいきれない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Surface, &["と"]), PARTICLE]),
            expect(&[(Reading, &["イイ"])]),
            expect(&[(Reading, &["キレ"]), VERB]),
            expect(&[NAI]),
        ],
    },
    Phrase {
        name: "ないとはいいきれない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Surface, &["と"]), PARTICLE]),
            expect(&[(Surface, &["は"]), PARTICLE]),
            expect(&[(Reading, &["イイ"])]),
            expect(&[(Reading, &["キレ"]), VERB]),
            expect(&[NAI]),
        ],
    },
    Phrase {
        name: "ないとはかぎらない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Surface, &["と"]), PARTICLE]),
            expect(&[(Surface, &["は"]), PARTICLE]),
            expect(&[(Reading, &["カギラ"]), VERB]),
            expect(&[NAI]),
        ],
    },
    Phrase {
        name: "ないともかぎらない",
        pattern: &[
            expect(&[NAI]),
            expect(&[(Surface, &["と"]), PARTICLE]),
            expect(&[(Surface, &["も"]), PARTICLE]),
            expect(&[(Reading, &["カギラ"]), VERB]),
            expect(&[NAI]),
        ],
    },
];

/// `ja-no-weak-phrase` の辞書。textlint-rule-ja-no-weak-phrase 2.0.0 の `lib/dict.js` から作った
pub const WEAK_PHRASES: [Phrase; 5] = [
    Phrase {
        name: "かも",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["かも"]),
                    (Pos, &["助詞"]),
                    (PosDetail1, &["副助詞"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["かも"]),
                    (Reading, &["カモ"]),
                    (Pronunciation, &["カモ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["。"]),
                    (Pos, &["記号"]),
                    (PosDetail1, &["句点"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["。"]),
                    (Reading, &["。"]),
                    (Pronunciation, &["。"]),
                ],
                skippable: false,
            },
        ],
    },
    Phrase {
        name: "かも",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["かも"]),
                    (Pos, &["助詞"]),
                    (PosDetail1, &["副助詞"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["かも"]),
                    (Reading, &["カモ"]),
                    (Pronunciation, &["カモ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["しれ"]),
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["一段"]),
                    (ConjugatedForm, &["連用形", "未然形"]),
                    (BasicForm, &["しれる"]),
                    (Reading, &["シレ"]),
                    (Pronunciation, &["シレ"]),
                ],
                skippable: false,
            },
        ],
    },
    Phrase {
        name: "思う",
        pattern: &[Expect {
            fields: &[
                (Surface, &["思う"]),
                (Pos, &["動詞"]),
                (PosDetail1, &["自立"]),
                (PosDetail2, &["*"]),
                (PosDetail3, &["*"]),
                (ConjugatedType, &["五段・ワ行促音便"]),
                (ConjugatedForm, &["基本形"]),
                (BasicForm, &["思う"]),
                (Reading, &["オモウ"]),
                (Pronunciation, &["オモウ"]),
            ],
            skippable: false,
        }],
    },
    Phrase {
        name: "思います",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["思い"]),
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["五段・ワ行促音便"]),
                    (ConjugatedForm, &["連用形"]),
                    (BasicForm, &["思う"]),
                    (Reading, &["オモイ"]),
                    (Pronunciation, &["オモイ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["ます"]),
                    (Pos, &["助動詞"]),
                    (PosDetail1, &["*"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["特殊・マス"]),
                    (ConjugatedForm, &["基本形"]),
                    (BasicForm, &["ます"]),
                    (Reading, &["マス"]),
                    (Pronunciation, &["マス"]),
                ],
                skippable: false,
            },
        ],
    },
    Phrase {
        name: "可能性を示唆している",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["可能"]),
                    (Pos, &["名詞"]),
                    (PosDetail1, &["形容動詞語幹"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["可能"]),
                    (Reading, &["カノウ"]),
                    (Pronunciation, &["カノー"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["性"]),
                    (Pos, &["名詞"]),
                    (PosDetail1, &["接尾"]),
                    (PosDetail2, &["一般"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["性"]),
                    (Reading, &["セイ"]),
                    (Pronunciation, &["セイ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["を"]),
                    (Pos, &["助詞"]),
                    (PosDetail1, &["格助詞"]),
                    (PosDetail2, &["一般"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["を"]),
                    (Reading, &["ヲ"]),
                    (Pronunciation, &["ヲ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["示唆"]),
                    (Pos, &["名詞"]),
                    (PosDetail1, &["サ変接続"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["示唆"]),
                    (Reading, &["シサ"]),
                    (Pronunciation, &["シサ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["し"]),
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["サ変・スル"]),
                    (ConjugatedForm, &["連用形"]),
                    (BasicForm, &["する"]),
                    (Reading, &["シ"]),
                    (Pronunciation, &["シ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["て"]),
                    (Pos, &["助詞"]),
                    (PosDetail1, &["接続助詞"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["て"]),
                    (Reading, &["テ"]),
                    (Pronunciation, &["テ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["いる"]),
                    (Pos, &["動詞"]),
                    (PosDetail1, &["非自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["一段"]),
                    (ConjugatedForm, &["基本形"]),
                    (BasicForm, &["いる"]),
                    (Reading, &["イル"]),
                    (Pronunciation, &["イル"]),
                ],
                skippable: false,
            },
        ],
    },
];

/// `ja-no-abusage` のうち、トークンの並びで見つける4項目。textlint-rule-ja-no-abusage 3.0.0 の `lib/dictionary.js` から作った。
/// `ずらい` の2項目は、並びの最初のトークンが動詞である
pub const ABUSAGES: [Phrase; 4] = [
    Phrase {
        name: "を適応",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["を"]),
                    (Pos, &["助詞"]),
                    (PosDetail1, &["格助詞"]),
                    (PosDetail2, &["一般"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["を"]),
                    (Reading, &["ヲ"]),
                    (Pronunciation, &["ヲ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["適応"]),
                    (Pos, &["名詞"]),
                    (PosDetail1, &["サ変接続"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["適応"]),
                    (Reading, &["テキオウ"]),
                    (Pronunciation, &["テキオー"]),
                ],
                skippable: false,
            },
        ],
    },
    Phrase {
        name: "可変する",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["可変"]),
                    (Pos, &["名詞"]),
                    (PosDetail1, &["一般"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["可変"]),
                    (Reading, &["カヘン"]),
                    (Pronunciation, &["カヘン"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Pos, &["動詞"]), (PosDetail1, &["自立"])],
                skippable: false,
            },
        ],
    },
    Phrase {
        name: "ずらい",
        pattern: &[
            Expect {
                fields: &[
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (ConjugatedForm, &["連用形"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["ず"])],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["らい"])],
                skippable: false,
            },
        ],
    },
    Phrase {
        name: "ずらい",
        pattern: &[
            Expect {
                fields: &[
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (ConjugatedForm, &["連用形"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["ずら"])],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["い"])],
                skippable: false,
            },
        ],
    },
];

/// `ja-no-abusage` のうち、文字列で見つける73件。textlint-rule-ja-no-abusage 3.0.0 の `dict/prh.yml` から作った。
/// `pattern` は `regex` crateの正規表現、`expected` は置き換え後の文字列で、`$1` などは一致したグループである
pub const ABUSAGE_WORDS: [Word; 73] = [
    Word {
        pattern: "値を返却する",
        expected: "値を返す",
    },
    Word {
        pattern: "(メソッド|関数)の返却値",
        expected: "$1の返り値",
    },
    Word {
        pattern: "例外を補足",
        expected: "例外を捕捉",
    },
    Word {
        pattern: "似て異なるもの",
        expected: "似て非なるもの",
    },
    Word {
        pattern: "愛苦しい",
        expected: "愛くるしい",
    },
    Word {
        pattern: "悪どい",
        expected: "あくどい",
    },
    Word {
        pattern: "(うんちく|薀蓄)を垂れる",
        expected: "$1を傾ける",
    },
    Word {
        pattern: "転稼",
        expected: "転嫁",
    },
    Word {
        pattern: "こんにちわ",
        expected: "こんにちは",
    },
    Word {
        pattern: "愛想を(振り|ふり)まく",
        expected: "愛嬌を$1まく",
    },
    Word {
        pattern: "論戦を張る",
        expected: "論陣を張る",
    },
    Word {
        pattern: "(いさぎ良い|潔よい)",
        expected: "潔い",
    },
    Word {
        pattern: "寸暇を惜しまず",
        expected: "寸暇を惜しんで",
    },
    Word {
        pattern: "寸暇を置かず",
        expected: "間を置かず",
    },
    Word {
        pattern: "一つ返事",
        expected: "二つ返事",
    },
    Word {
        pattern: "怒り心頭に達する",
        expected: "怒り心頭に発する",
    },
    Word {
        pattern: "熱にうなされる",
        expected: "熱に浮かされる",
    },
    Word {
        pattern: "やるせぬ",
        expected: "やるせない",
    },
    Word {
        pattern: "馬脚を(晒す|さらす|出す)",
        expected: "馬脚を露わす",
    },
    Word {
        pattern: "雪辱を晴らす",
        expected: "雪辱を果たす",
    },
    Word {
        pattern: "うる覚え",
        expected: "うろ覚え",
    },
    Word {
        pattern: "頭を(傾げる|かしげる|かたげる)",
        expected: "首を$1",
    },
    Word {
        pattern: "口先三寸",
        expected: "舌先三寸",
    },
    Word {
        pattern: "舌の先の乾かぬ",
        expected: "舌の根の乾かぬ",
    },
    Word {
        pattern: "悪評(?:嘖々|嘖嘖|さくさく)",
        expected: "悪評高い",
    },
    Word {
        pattern: "器管",
        expected: "器官",
    },
    Word {
        pattern: "(火蓋|火ぶた)を(?:切って)?落とす",
        expected: "$1を切る",
    },
    Word {
        pattern: "(火蓋|火ぶた)が(?:切って)?落とされる",
        expected: "$1が切られる",
    },
    Word {
        pattern: "(取り付く|とりつく)(?:暇|ヒマ|ひま)(も|が)(無い|ない)",
        expected: "$1島$2$3",
    },
    Word {
        pattern: "上(?:や|へ)下への",
        expected: "上を下への",
    },
    Word {
        pattern: "押しも押されぬ",
        expected: "押しも押されもせぬ",
    },
    Word {
        pattern: "掻き入れ",
        expected: "書き入れ",
    },
    Word {
        pattern: "引くに引かれない",
        expected: "引くに引けない",
    },
    Word {
        pattern: "死ぬに死なれない",
        expected: "死ぬに死ねない",
    },
    Word {
        pattern: "腹が煮えくり返る",
        expected: "腸が煮えくり返る",
    },
    Word {
        pattern: "胸先三寸",
        expected: "胸三寸",
    },
    Word {
        pattern: "肝に据えかねる",
        expected: "腹に据えかねる",
    },
    Word {
        pattern: "鼻にも(?:かけない|掛けない)",
        expected: "鼻も引っ掛けない",
    },
    Word {
        pattern: "へそを抱えて",
        expected: "腹を抱えて",
    },
    Word {
        pattern: "後ろ立て",
        expected: "後ろ盾",
    },
    Word {
        pattern: "絶対絶命",
        expected: "絶体絶命",
    },
    Word {
        pattern: "有頂点",
        expected: "有頂天",
    },
    Word {
        pattern: "(自分本位|自[己分]本意)",
        expected: "自己本位",
    },
    Word {
        pattern: "一発即発",
        expected: "一触即発",
    },
    Word {
        pattern: "意気高々",
        expected: "意気揚々",
    },
    Word {
        pattern: "青色吐息",
        expected: "青息吐息",
    },
    Word {
        pattern: "固定概念",
        expected: "固定観念",
    },
    Word {
        pattern: "目(に|にも|が)止ま([らろりっるれ])",
        expected: "目$1留ま$2",
    },
    Word {
        pattern: "目を止め",
        expected: "目を留め",
    },
    Word {
        pattern: "無理([のが])効かない",
        expected: "無理$1利かない",
    },
    Word {
        pattern: "睨みを効かせ",
        expected: "睨みを利かせ",
    },
    Word {
        pattern: "([気癪])に触る",
        expected: "$1に障る",
    },
    Word {
        pattern: "見るに[耐絶]えない",
        expected: "見るに堪えない",
    },
    Word {
        pattern: "頭に留め[て]?[置お]く",
        expected: "頭に入れておく",
    },
    Word {
        pattern: "(?:関心|感心)を買う",
        expected: "歓心を買う",
    },
    Word {
        pattern: "恩を着る",
        expected: "恩に着る",
    },
    Word {
        pattern: "脚光を集め",
        expected: "脚光を浴び",
    },
    Word {
        pattern: "口が[固硬]い",
        expected: "口が堅い",
    },
    Word {
        pattern: "思いも([付つ])かない",
        expected: "思い$1かない",
    },
    Word {
        pattern: "足り([得え])ない",
        expected: "たり$1ない",
    },
    Word {
        pattern: "案の[上条]",
        expected: "案の定",
    },
    Word {
        pattern: "合いの手を打つ",
        expected: "合いの手を入れる",
    },
    Word {
        pattern: "相(づち|槌)を入れる",
        expected: "相$1を打つ",
    },
    Word {
        pattern: "耳障り([のが])?([良よ]い)",
        expected: "耳当たり$1$2",
    },
    Word {
        pattern: "無理なからぬ",
        expected: "無理からぬ",
    },
    Word {
        pattern: "間髪([い容]れず|[を]?[お置]かず)",
        expected: "間髪をいれず",
    },
    Word {
        pattern: "食指[をが](そそる|そそられる|伸ばす)",
        expected: "食指が動く",
    },
    Word {
        pattern: "念頭に(入れる|する)",
        expected: "念頭におく",
    },
    Word {
        pattern: "嫌気が(して|した)",
        expected: "嫌気が差$1",
    },
    Word {
        pattern: "精魂(が)?尽き",
        expected: "精根$1尽き",
    },
    Word {
        pattern: "苦虫を[か噛]んだような",
        expected: "苦虫を噛み潰したような",
    },
    Word {
        pattern: "喝(を)?入れ",
        expected: "活$1入れ",
    },
    Word {
        pattern: "一同に会する",
        expected: "一堂に会する",
    },
];

/// `ja-no-redundant-expression` の辞書。textlint-rule-ja-no-redundant-expression 4.0.1 の `lib/dictionary.js` から作った。
/// `allows` は、一致したトークンの表層をつなげた文字列に当てる既定のパターンで、`js_regex::pattern` で読む
pub const REDUNDANTS: [Redundant; 6] = [
    Redundant {
        id: "dict1",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["する"]),
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["サ変・スル"]),
                    (ConjugatedForm, &["基本形"]),
                    (BasicForm, &["する"]),
                    (Reading, &["スル"]),
                    (Pronunciation, &["スル"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Pos, &["名詞"]), (Reading, &["コト"])],
                skippable: false,
            },
            Expect {
                fields: &[(Pos, &["助詞"])],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[(BasicForm, &["可", "可能", "不可能", "不能", "不可"])],
                skippable: false,
            },
            Expect {
                fields: &[(Pos, &["助動詞"])],
                skippable: false,
            },
        ],
        allows: &[],
    },
    Redundant {
        id: "dict2",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["する"]),
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["サ変・スル"]),
                    (ConjugatedForm, &["基本形"]),
                    (BasicForm, &["する"]),
                    (Reading, &["スル"]),
                    (Pronunciation, &["スル"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Pos, &["名詞"]), (Reading, &["コト"])],
                skippable: false,
            },
            Expect {
                fields: &[(Pos, &["助詞"])],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (BasicForm, &["できる"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[],
                skippable: false,
            },
        ],
        allows: &[],
    },
    Redundant {
        id: "dict3",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["で"]),
                    (Pos, &["助動詞"]),
                    (PosDetail1, &["*"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["特殊・ダ"]),
                    (ConjugatedForm, &["連用形"]),
                    (BasicForm, &["だ"]),
                    (Reading, &["デ"]),
                    (Pronunciation, &["デ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Reading, &["アル"])],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[(Surface, &["と"]), (Pos, &["助詞"])],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["一段"]),
                    (ConjugatedForm, &["連用形"]),
                    (Reading, &["イエ"]),
                    (Pronunciation, &["イエ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["ます"]),
                    (Pos, &["助動詞"]),
                    (PosDetail1, &["*"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["特殊・マス"]),
                    (ConjugatedForm, &["基本形"]),
                    (BasicForm, &["ます"]),
                    (Reading, &["マス"]),
                    (Pronunciation, &["マス"]),
                ],
                skippable: false,
            },
        ],
        allows: &[],
    },
    Redundant {
        id: "dict4",
        pattern: &[
            Expect {
                fields: &[
                    (Surface, &["で"]),
                    (Pos, &["助動詞"]),
                    (PosDetail1, &["*"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["特殊・ダ"]),
                    (ConjugatedForm, &["連用形"]),
                    (BasicForm, &["だ"]),
                    (Reading, &["デ"]),
                    (Pronunciation, &["デ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Reading, &["アル"])],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[(Surface, &["と"]), (Pos, &["助詞"])],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[
                    (Surface, &["考え"]),
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["一段"]),
                    (ConjugatedForm, &["連用形"]),
                    (BasicForm, &["考える"]),
                    (Reading, &["カンガエ"]),
                    (Pronunciation, &["カンガエ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["て"]),
                    (Pos, &["助詞"]),
                    (PosDetail1, &["接続助詞"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["て"]),
                    (Reading, &["テ"]),
                    (Pronunciation, &["テ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[
                    (Pos, &["動詞"]),
                    (PosDetail1, &["非自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (BasicForm, &["いる"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["ます"]),
                    (Pos, &["助動詞"]),
                    (ConjugatedType, &["特殊・マス"]),
                    (ConjugatedForm, &["基本形"]),
                    (BasicForm, &["ます"]),
                    (Reading, &["マス"]),
                    (Pronunciation, &["マス"]),
                ],
                skippable: true,
            },
        ],
        allows: &[],
    },
    Redundant {
        id: "dict5",
        pattern: &[
            Expect {
                fields: &[(Pos, &["名詞"]), (PosDetail1, &["サ変接続"])],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["を"]),
                    (Pos, &["助詞"]),
                    (PosDetail1, &["格助詞"]),
                    (PosDetail2, &["一般"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["を"]),
                    (Reading, &["ヲ"]),
                    (Pronunciation, &["ヲ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[
                    (Pos, &["動詞"]),
                    (PosDetail1, &["自立"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["五段・ワ行促音便"]),
                    (BasicForm, &["行う"]),
                ],
                skippable: false,
            },
        ],
        allows: &[
            "/^処理を行[ぁ-ん]/",
            "/^[ァ-ヶ]+を.?行[ぁ-ん]/",
            "/^[a-zA-Z]+を.?行[ぁ-ん]/",
        ],
    },
    Redundant {
        id: "dict6",
        pattern: &[
            Expect {
                fields: &[(Pos, &["名詞"]), (PosDetail1, &["サ変接続"])],
                skippable: false,
            },
            Expect {
                fields: &[
                    (Surface, &["を"]),
                    (Pos, &["助詞"]),
                    (PosDetail1, &["格助詞"]),
                    (PosDetail2, &["一般"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["を"]),
                    (Reading, &["ヲ"]),
                    (Pronunciation, &["ヲ"]),
                ],
                skippable: false,
            },
            Expect {
                fields: &[(Surface, &["、", "､", "，", ","])],
                skippable: true,
            },
            Expect {
                fields: &[
                    (Surface, &["実行"]),
                    (Pos, &["名詞"]),
                    (PosDetail1, &["サ変接続"]),
                    (PosDetail2, &["*"]),
                    (PosDetail3, &["*"]),
                    (ConjugatedType, &["*"]),
                    (ConjugatedForm, &["*"]),
                    (BasicForm, &["実行"]),
                    (Reading, &["ジッコウ"]),
                    (Pronunciation, &["ジッコー"]),
                ],
                skippable: false,
            },
        ],
        allows: &[
            "/^処理を実行/",
            "/^[ァ-ヶ]+を.?実行/",
            "/^[a-zA-Z]+を.?実行/",
        ],
    },
];

/// `no-ai-words` の辞書。textlint-rule-preset-ai-words-ja 1.2.1 の `src/dictionary.ts` から作った。
/// `name` は、textlintの文面の見出しの語である
pub const AI_WORDS: [Phrase; 54] = [
    Phrase {
        name: "効く",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["効く"]),
        ])],
    },
    Phrase {
        name: "壊れる",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["壊れる"]),
        ])],
    },
    Phrase {
        name: "走る",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (Surface, &["走る", "走っ", "走り", "走ら", "走れ", "走ろ"]),
        ])],
    },
    Phrase {
        name: "焼く",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["焼く"]),
        ])],
    },
    Phrase {
        name: "黙って",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["黙る"]),
        ])],
    },
    Phrase {
        name: "〜に落ちる",
        pattern: &[
            expect(&[
                (Pos, &["助詞"]),
                (PosDetail1, &["格助詞"]),
                (Surface, &["に"]),
            ]),
            expect(&[
                (Pos, &["動詞"]),
                (PosDetail1, &["自立"]),
                (BasicForm, &["落ちる"]),
            ]),
        ],
    },
    Phrase {
        name: "崩す",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["崩す"]),
        ])],
    },
    Phrase {
        name: "値を動かす",
        pattern: &[
            expect(&[(Pos, &["名詞"]), (Surface, &["値"])]),
            expect(&[(Pos, &["助詞"]), (Surface, &["を"])]),
            expect(&[
                (Pos, &["動詞"]),
                (PosDetail1, &["自立"]),
                (BasicForm, &["動かす"]),
            ]),
        ],
    },
    Phrase {
        name: "〜から引く",
        pattern: &[
            expect(&[
                (Pos, &["助詞"]),
                (PosDetail1, &["格助詞"]),
                (Surface, &["から"]),
            ]),
            expect(&[
                (Pos, &["動詞"]),
                (PosDetail1, &["自立"]),
                (BasicForm, &["引く"]),
            ]),
        ],
    },
    Phrase {
        name: "経路",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["経路"])])],
    },
    Phrase {
        name: "死活",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["死活"])])],
    },
    Phrase {
        name: "漏れ",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["漏れ"])])],
    },
    Phrase {
        name: "帰結",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["帰結"])])],
    },
    Phrase {
        name: "原初",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["原初"])])],
    },
    Phrase {
        name: "穴",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["穴"])])],
    },
    Phrase {
        name: "無差別",
        pattern: &[
            expect(&[(Pos, &["接頭詞"]), (Surface, &["無"])]),
            expect(&[(Pos, &["名詞"]), (BasicForm, &["差別"])]),
        ],
    },
    Phrase {
        name: "正本",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["正本"])])],
    },
    Phrase {
        name: "部品",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["部品"])])],
    },
    Phrase {
        name: "検査",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["検査"])])],
    },
    Phrase {
        name: "正典",
        pattern: &[
            expect(&[(Pos, &["接頭詞"]), (Surface, &["正"])]),
            expect(&[(Pos, &["名詞"]), (Surface, &["典"])]),
        ],
    },
    Phrase {
        name: "配線する",
        pattern: &[
            expect(&[
                (Pos, &["助詞"]),
                (PosDetail1, &["格助詞"]),
                (Surface, &["に"]),
            ]),
            expect(&[(Pos, &["名詞"]), (BasicForm, &["配線"])]),
            expect(&[
                (Pos, &["動詞"]),
                (PosDetail1, &["自立"]),
                (BasicForm, &["する"]),
            ]),
        ],
    },
    Phrase {
        name: "太る",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["太る"]),
        ])],
    },
    Phrase {
        name: "見張る",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["見張る"]),
        ])],
    },
    Phrase {
        name: "原料",
        pattern: &[expect(&[(Pos, &["名詞"]), (Surface, &["原料"])])],
    },
    Phrase {
        name: "実測",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["実測"])])],
    },
    Phrase {
        name: "疑う",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["疑う"]),
        ])],
    },
    Phrase {
        name: "照合",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["照合"])])],
    },
    Phrase {
        name: "見落とす",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["見落とす"]),
        ])],
    },
    Phrase {
        name: "突き合わせる",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["突き合わせる"]),
        ])],
    },
    Phrase {
        name: "断定",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["断定"])])],
    },
    Phrase {
        name: "入口",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["入口"])])],
    },
    Phrase {
        name: "土台",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["土台"])])],
    },
    Phrase {
        name: "道具",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["道具"])])],
    },
    Phrase {
        name: "核心",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["核心"])])],
    },
    Phrase {
        name: "構図",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["構図"])])],
    },
    Phrase {
        name: "線引き",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["線引き"])])],
    },
    Phrase {
        name: "事故",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["事故"])])],
    },
    Phrase {
        name: "混ざる",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["混ざる"]),
        ])],
    },
    Phrase {
        name: "落とし穴",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["落とし穴"])])],
    },
    Phrase {
        name: "破綻",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["破綻"])])],
    },
    Phrase {
        name: "素通り",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["素通り"])])],
    },
    Phrase {
        name: "切り分ける",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["切り分ける"]),
        ])],
    },
    Phrase {
        name: "潰す",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["潰す"]),
        ])],
    },
    Phrase {
        name: "踏み込む",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["踏み込む"]),
        ])],
    },
    Phrase {
        name: "溶かす",
        pattern: &[expect(&[
            (Pos, &["動詞"]),
            (PosDetail1, &["自立"]),
            (BasicForm, &["溶かす"]),
        ])],
    },
    Phrase {
        name: "既定では",
        pattern: &[
            expect(&[(Pos, &["名詞"]), (BasicForm, &["既定"])]),
            expect(&[
                (Pos, &["助詞"]),
                (PosDetail1, &["格助詞"]),
                (Surface, &["で"]),
            ]),
        ],
    },
    Phrase {
        name: "定番",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["定番"])])],
    },
    Phrase {
        name: "定石",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["定石"])])],
    },
    Phrase {
        name: "〜した瞬間",
        pattern: &[
            expect(&[(Pos, &["助動詞"]), (Surface, &["た"])]),
            expect(&[(Pos, &["名詞"]), (BasicForm, &["瞬間"])]),
        ],
    },
    Phrase {
        name: "静かに",
        pattern: &[
            expect(&[(Pos, &["名詞"]), (Surface, &["静か"])]),
            expect(&[
                (Pos, &["助詞"]),
                (PosDetail1, &["副詞化"]),
                (Surface, &["に"]),
            ]),
            expect(&[
                (Pos, &["動詞"]),
                (PosDetail1, &["自立"]),
                (
                    BasicForm,
                    &[
                        "壊れる",
                        "落ちる",
                        "止まる",
                        "消える",
                        "捨てる",
                        "失う",
                        "漏れる",
                    ],
                ),
            ]),
        ],
    },
    Phrase {
        name: "静かに",
        pattern: &[
            expect(&[(Pos, &["名詞"]), (Surface, &["静か"])]),
            expect(&[
                (Pos, &["助詞"]),
                (PosDetail1, &["副詞化"]),
                (Surface, &["に"]),
            ]),
            expect(&[(Pos, &["名詞"]), (PosDetail1, &["サ変接続"])]),
            expect(&[
                (Pos, &["動詞"]),
                (PosDetail1, &["自立"]),
                (BasicForm, &["する"]),
            ]),
            expect(&[
                (Pos, &["動詞"]),
                (PosDetail1, &["接尾"]),
                (BasicForm, &["れる", "られる"]),
            ]),
        ],
    },
    Phrase {
        name: "無言",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["無言"])])],
    },
    Phrase {
        name: "門",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["門"])])],
    },
    Phrase {
        name: "ゲート",
        pattern: &[expect(&[(Pos, &["名詞"]), (BasicForm, &["ゲート"])])],
    },
];
