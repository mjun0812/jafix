//! `no-dead-link` の外部のURLの確認。textlintの `isAliveURI` と同じく、`HEAD` で始め、2xxでなければ `GET` で
//! 再試行し、リダイレクトを1回ずつ辿る

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use url::Url;

use crate::rules::builtin::NoDeadLink;

/// 1回の要求を、接続から本文の読み終わりまでで打ち切る時間
const TIMEOUT: Duration = Duration::from_secs(10);

/// 1回の `jafix check` で共有する、外部のURLの確認器と確認の結果
pub struct UrlChecker {
    /// リダイレクトを辿らず、2xx以外の応答も誤りにしないclient
    #[allow(dead_code)] // skeleton: not implemented
    agent: ureq::Agent,
    /// 確かめたURLと最初の方式の組ごとの結果。別のファイルから同じ組を同時に確かめても、要求は1組の分だけにする
    #[allow(dead_code)] // skeleton: not implemented
    results: Mutex<HashMap<(Url, Method), Pending>>,
}

/// 1組の確認の結果。最初に確かめるスレッドが埋め、同じ組を待つスレッドはそれを読む
type Pending = Arc<OnceLock<Outcome>>;

impl Default for UrlChecker {
    fn default() -> Self {
        Self {
            agent: ureq::Agent::config_builder()
                .max_redirects(0)
                .http_status_as_error(false)
                .timeout_global(Some(TIMEOUT))
                .build()
                .into(),
            results: Mutex::default(),
        }
    }
}

/// 要求の方式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)] // skeleton: not implemented
enum Method {
    Head,
    Get,
}

/// 1つのURLを確かめた結果
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)] // skeleton: not implemented
pub enum Outcome {
    /// 2xxの応答が返った
    Alive,
    /// 届かない。`reason` は `404 Not Found` のような状態か、接続の誤りの理由である
    Dead { reason: String },
    /// リダイレクトを辿った先が2xxだった。`status` は最初の応答の状態である
    Redirected { to: Url, status: String },
}

impl UrlChecker {
    /// `urls` を、最大 `rule.concurrency` 個ずつ同時に確かめ、同じ順に結果を返す。前に確かめたURLと最初の方式の組は、
    /// その結果を使う
    #[allow(dead_code)] // skeleton: not implemented
    pub fn check(&self, urls: &[Url], rule: &NoDeadLink) -> Vec<Outcome> {
        let _ = (urls, rule);
        todo!("skeleton: not implemented")
    }
}
