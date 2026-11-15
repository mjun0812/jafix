//! prh 6.1.0の辞書の読み込みと照合。文字列の中の置き換えの箇所を返すだけで、どの本文の範囲に当てるかと、
//! 箇所を修正にするか指摘にするかは呼び出す側が決める

// skeleton: not implemented (実装したら、この行と次の行を消す)
#![allow(dead_code)]

use std::ops::Range;
use std::path::PathBuf;

use regress::Regex;

/// 辞書の出どころ
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// YAMLのファイル。`imports` の相対パスは、このファイルのあるディレクトリから解決する
    Path(PathBuf),
    /// YAMLの内容と、その `imports` の相対パスを解決するディレクトリ
    Contents { yaml: String, base: PathBuf },
}

/// 出どころの一覧から読んで1つにまとめた辞書
#[derive(Debug)]
pub struct Dictionary {
    /// まとめた後のルール。同じ `expected` のルールは、先にあるものだけを持つ
    rules: Vec<Rule>,
}

/// 辞書の1つのルール
#[derive(Debug)]
pub struct Rule {
    /// 置き換え後の文字列。`$1`〜`$99` で捕捉グループを参照する
    pub expected: String,
    /// ルールの `prh` に書いた補足
    pub note: Option<String>,
    /// ルールを読んだ出どころの、`Dictionary::load` に渡した一覧での番号。`imports` で取り込んだルールは、取り込んだ側の番号である
    pub source: usize,
    regex: Regex,
    /// `regexpMustEmpty` のグループの番号
    must_empty: Option<usize>,
}

/// 置き換えの1箇所
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replacement {
    /// 照合した文字列の中のバイト範囲
    pub range: Range<usize>,
    /// 置き換え後の文字列
    pub text: String,
    /// `Dictionary::rules` での番号
    pub rule: usize,
}

/// 辞書を読めない理由と、その出どころの番号
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadError {
    pub source: usize,
    pub reason: String,
}

impl Dictionary {
    /// `sources` を順に読み、1つにまとめる。各ルールの正規表現と `specs` を、ここで作って確かめる
    pub fn load(sources: &[Source]) -> Result<Self, LoadError> {
        let _ = sources;
        todo!("skeleton: not implemented")
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// `text` の中の置き換えの箇所を、重なりを解いて位置の順に返す
    pub fn replacements(&self, text: &str) -> Vec<Replacement> {
        let _ = text;
        todo!("skeleton: not implemented")
    }
}
