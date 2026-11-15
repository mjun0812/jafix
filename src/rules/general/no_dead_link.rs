//! `no-dead-link`。リンクの宛先のファイルかURLがあるかを確かめる。ローカルのファイルはhookでも確かめ、`http:`・
//! `https:` のURLは `jafix check` だけが接続して確かめる

mod external;

use std::borrow::Cow;
use std::path::PathBuf;

use url::Url;

use super::Links;
use crate::filter::Ignored;
use crate::prose::{Document, LinkDestination};
use crate::rules::builtin::NoDeadLink;
use crate::rules::{Report, RuleId};

use self::external::Outcome;
pub use self::external::UrlChecker;

/// 宛先を解決した先
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
    document: &Document,
    rule: &NoDeadLink,
    links: &Links,
    ignored: &Ignored,
) -> Vec<Report> {
    let report = |link: &LinkDestination, message: String| Report {
        range: link.range.clone(),
        rule: RuleId::NoDeadLink.name(),
        message: Cow::Owned(message),
    };
    let mut reports = Vec::new();
    let mut external = Vec::new();
    for link in &document.links {
        if link.blockquote
            || rule.ignore.is_match(&link.url)
            || ignored.covers(&link.range, RuleId::NoDeadLink.name())
        {
            continue;
        }
        match resolve(&link.url, rule, links) {
            Some(Target::Local(path)) if !path.exists() => {
                reports.push(report(
                    link,
                    format!("リンク先のファイルが無い: {}", link.url),
                ));
            }
            Some(Target::External(url)) => external.push((link, url)),
            Some(Target::Local(_)) | None => {}
        }
    }
    let Some(checker) = links.urls else {
        return reports;
    };
    let urls: Vec<Url> = external.iter().map(|(_, url)| url.clone()).collect();
    for ((link, url), outcome) in external.iter().zip(checker.check(&urls, rule)) {
        // 相対の宛先を `baseURI` で解決したURLは、書いた宛先ではなく解決したURLを見せる
        let shown = if Url::parse(&link.url).is_ok() {
            link.url.as_str()
        } else {
            url.as_str()
        };
        match outcome {
            Outcome::Alive => {}
            Outcome::Dead { reason } => {
                reports.push(report(
                    link,
                    format!("リンク先に届かない: {shown} ({reason})"),
                ));
            }
            Outcome::Redirected { to, status } => reports.push(report(
                link,
                format!("リンク先が {to} へリダイレクトする ({status})。宛先を {to} にする"),
            )),
        }
    }
    reports
}

/// 宛先の文字列 `destination` を、`rule` の `baseURI`・`httpOnly`・`checkRelative` と、Markdownのファイル `links.file`
/// に従って解決する。確かめない宛先 (`http:`・`https:`・`file:` 以外の方式、`checkRelative` が偽の相対の宛先、
/// `httpOnly` が真で `baseURI` が無いときの相対の宛先と絶対パス) なら `None`
fn resolve(destination: &str, rule: &NoDeadLink, links: &Links) -> Option<Target> {
    let url = match Url::parse(destination) {
        Ok(url) => url,
        Err(_) if destination.starts_with('/') => {
            if rule.http_only {
                rule.base_uri.as_ref()?.join(destination).ok()?
            } else {
                // textlintと同じく、パーセント符号化を解かずにそのままパスとして確かめる
                let path = destination.split(['?', '#']).next().unwrap_or_default();
                return Some(Target::Local(PathBuf::from(path)));
            }
        }
        Err(_) => {
            if !rule.check_relative {
                return None;
            }
            let base = match &rule.base_uri {
                Some(base) => base.clone(),
                None if rule.http_only => return None,
                None => Url::from_file_path(std::path::absolute(links.file).ok()?).ok()?,
            };
            base.join(destination).ok()?
        }
    };
    match url.scheme() {
        "http" | "https" => Some(Target::External(url)),
        "file" => {
            let mut url = url;
            url.set_query(None);
            url.set_fragment(None);
            url.to_file_path().ok().map(Target::Local)
        }
        _ => None,
    }
}
