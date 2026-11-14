//! 形態素解析を使う11ルール

// skeleton: not implemented (実装したら、この行と次の行を消す)
#![allow(dead_code)]

mod dict;
mod pattern;
mod projection;

use crate::prose::Document;

use super::Report;
use super::builtin::Builtin;

/// 有効なルールの指摘を返す。順は問わない (`rules::report` が位置の順に並べる)
pub fn report(src: &str, document: &Document, builtin: &Builtin) -> Vec<Report<'static>> {
    let _ = (src, document, builtin);
    todo!("skeleton: not implemented")
}
