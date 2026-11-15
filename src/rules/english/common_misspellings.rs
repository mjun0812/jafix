//! `common-misspellings` (textlint-rule-common-misspellings 1.0.1)。Wikipediaの誤綴りの一覧 (CC BY-SA 3.0) の誤綴りを
//! 本文の範囲に当て、正しい語を示して指摘する。textlint-rule-common-misspellingsと `misspellings` のコードと一覧はGPLなので
//! 写さず、判定の条件から書き直し、一覧はWikipediaのページの固定したrevisionから作った

use crate::prose::Prose;
use crate::rules::Report;
use crate::rules::builtin::CommonMisspellings;

/// 先頭の `#` の行が出典とライセンスで、後は1行1項目の `<小文字の誤綴り>\t<正しい語>,<正しい語>…` である
#[allow(dead_code)] // skeleton: not implemented
static LIST: &str = include_str!("data/common-misspellings.txt");

/// 引用の中にない本文の範囲に、一覧の誤綴りのすべてを選択にした `\b(?:…)\b` を大文字と小文字を区別せずに当てる。
/// 同じ位置では最も長い一致を取り、一致を小文字にして一覧を引き、`ignore` に無ければ、正しい語のすべてに一致の大文字を
/// 写して文面に含めた指摘にする。範囲は一致の全体である
pub fn report(src: &str, texts: &[Prose], rule: &CommonMisspellings) -> Vec<Report> {
    let _ = (src, texts, rule);
    todo!("skeleton: not implemented")
}
