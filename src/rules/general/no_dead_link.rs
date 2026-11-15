//! `no-dead-link`。リンクの宛先のファイルかURLがあるかを確かめる。ローカルのファイルはhookでも確かめ、`http:`・
//! `https:` のURLは `jafix check` だけが接続して確かめる

mod external;

use std::path::PathBuf;

use url::Url;

use super::Links;
use crate::filter::Ignored;
use crate::prose::Document;
use crate::rules::Report;
use crate::rules::builtin::NoDeadLink;

pub use self::external::UrlChecker;

/// 宛先を解決した先
#[allow(dead_code)] // skeleton: not implemented
enum Target {
    /// ローカルのファイルかディレクトリ。`?` と `#` から後ろを除いたパスである
    Local(PathBuf),
    /// `http:`・`https:` のURL
    External(Url),
}

/// `document` のリンクの宛先のうち、引用の外にあり、`ignore` に当たらず、`ignored` に含まれないものを解決して
/// 確かめ、ファイルが無いか、URLが届かないかリダイレクトする宛先の指摘を返す。`links.urls` が無ければ、`http:`・
/// `https:` に解決した宛先を確かめない
pub fn report(
    src: &str,
    document: &Document,
    rule: &NoDeadLink,
    links: &Links,
    ignored: &Ignored,
) -> Vec<Report> {
    let _ = (src, document, rule, links, ignored, resolve);
    todo!("skeleton: not implemented")
}

/// 宛先の文字列 `destination` を、`rule` の `baseURI`・`httpOnly`・`checkRelative` と、Markdownのファイル `links.file`
/// に従って解決する。確かめない宛先 (`http:`・`https:`・`file:` 以外の方式、`checkRelative` が偽の相対の宛先、
/// `httpOnly` が真で `baseURI` が無いときの相対の宛先と絶対パス) なら `None`
fn resolve(destination: &str, rule: &NoDeadLink, links: &Links) -> Option<Target> {
    let _ = (destination, rule, links);
    todo!("skeleton: not implemented")
}
