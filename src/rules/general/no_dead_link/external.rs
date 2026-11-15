//! `no-dead-link` の外部のURLの確認。textlintの `isAliveURI` と同じく、`HEAD` で始め、2xxでなければ `GET` で
//! 再試行し、リダイレクトを1回ずつ辿る

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use ureq::http::{Response, StatusCode};
use url::Url;

use crate::rules::builtin::NoDeadLink;

/// 1回の要求を、接続から本文の読み終わりまでで打ち切る時間
const TIMEOUT: Duration = Duration::from_secs(10);

/// textlintと同じく、リダイレクトを辿る回数の上限
const MAX_REDIRECTS: usize = 20;

/// 1回の `jafix check` で共有する、外部のURLの確認器と確認の結果
pub struct UrlChecker {
    /// リダイレクトを辿らず、2xx以外の応答も誤りにしないclient
    agent: ureq::Agent,
    /// 確かめたURLと最初の方式の組ごとの結果。別のファイルから同じ組を同時に確かめても、要求は1組の分だけにする
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
enum Method {
    Head,
    Get,
}

/// 1つのURLを確かめた結果
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub fn check(&self, urls: &[Url], rule: &NoDeadLink) -> Vec<Outcome> {
        let next = AtomicUsize::new(0);
        let mut outcomes = vec![Outcome::Alive; urls.len()];
        std::thread::scope(|scope| {
            let workers: Vec<_> = (0..rule.concurrency.min(urls.len()))
                .map(|_| {
                    scope.spawn(|| {
                        let mut done = Vec::new();
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some(url) = urls.get(index) else {
                                return done;
                            };
                            done.push((index, self.outcome(url, rule)));
                        }
                    })
                })
                .collect();
            for worker in workers {
                for (index, outcome) in worker.join().expect("確認のスレッドはpanicしない")
                {
                    outcomes[index] = outcome;
                }
            }
        });
        outcomes
    }

    /// `url` の確認の結果。同じURLと最初の方式の組を前に確かめていれば、その結果を待って使う。`#` から後ろは
    /// サーバーに送らないので、それだけが違うURLは同じURLとして確かめ、`#` の無いリダイレクト先に付け直す
    fn outcome(&self, url: &Url, rule: &NoDeadLink) -> Outcome {
        let method = if rule.prefer_get.contains(&url.origin()) {
            Method::Get
        } else {
            Method::Head
        };
        let mut target = url.clone();
        target.set_fragment(None);
        let pending = Arc::clone(
            self.results
                .lock()
                .expect("結果の表を持つスレッドはpanicしない")
                .entry((target.clone(), method))
                .or_default(),
        );
        let mut outcome = pending
            .get_or_init(|| self.probe(&target, method, 0, 0, rule))
            .clone();
        if let Outcome::Redirected { to, .. } = &mut outcome {
            inherit_fragment(to, url);
        }
        outcome
    }

    /// `url` を `method` で確かめる。`retries` はこれまでの再試行の回数、`redirects` はこれまでに辿ったリダイレクトの数である
    fn probe(
        &self,
        url: &Url,
        method: Method,
        retries: usize,
        redirects: usize,
        rule: &NoDeadLink,
    ) -> Outcome {
        let response = match self.request(url, method, rule) {
            Ok(response) => response,
            Err(_) if retries < rule.retry => {
                return self.probe(url, Method::Get, retries + 1, redirects, rule);
            }
            Err(e) => {
                return Outcome::Dead {
                    reason: e.to_string(),
                };
            }
        };
        let status = response.status();
        if matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308) {
            // textlintと同じく、リダイレクトを無視する設定では、辿った先が届かなくても指摘しない
            if rule.ignore_redirects {
                return Outcome::Alive;
            }
            let dead = Outcome::Dead {
                reason: status_text(status),
            };
            let Some(location) = response
                .headers()
                .get("location")
                .and_then(|location| location.to_str().ok())
                .and_then(|location| url.join(location).ok())
            else {
                return dead;
            };
            if redirects >= MAX_REDIRECTS {
                return dead;
            }
            let mut to = match self.probe(&location, method, retries, redirects + 1, rule) {
                Outcome::Alive => location,
                Outcome::Redirected { to, .. } => to,
                Outcome::Dead { .. } => return dead,
            };
            inherit_fragment(&mut to, url);
            return Outcome::Redirected {
                to,
                status: status_text(status),
            };
        }
        if status.is_success() {
            return Outcome::Alive;
        }
        if retries >= rule.retry {
            return Outcome::Dead {
                reason: status_text(status),
            };
        }
        if method == Method::Get {
            std::thread::sleep(retry_wait(&response, retries, rule));
        }
        self.probe(url, Method::Get, retries + 1, redirects, rule)
    }

    fn request(
        &self,
        url: &Url,
        method: Method,
        rule: &NoDeadLink,
    ) -> Result<Response<ureq::Body>, ureq::Error> {
        let request = match method {
            Method::Head => self.agent.head(url.as_str()),
            Method::Get => self.agent.get(url.as_str()),
        };
        request
            .header("User-Agent", &rule.user_agent)
            .header("Accept", "*/*")
            .call()
    }
}

/// RFC 7231 §7.1.2と同じく、リダイレクト先 `to` に `#` が無ければ、リダイレクトの元の `from` の `#` から後ろを引き継ぐ
fn inherit_fragment(to: &mut Url, from: &Url) {
    if to.fragment().is_none() {
        to.set_fragment(from.fragment());
    }
}

/// `GET` で2xxでなかった応答の後、再試行するまで待つ時間。`Retry-After` の秒数が `maxRetryAfterTime` 以下ならその秒数、
/// `Retry-After` が無ければ `retries ** 2 * 100` ミリ秒が `maxRetryTime` 以下ならその時間で、上限を超えれば待たない
fn retry_wait(response: &Response<ureq::Body>, retries: usize, rule: &NoDeadLink) -> Duration {
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok()?.trim().parse::<u64>().ok());
    let (wait, max) = match retry_after {
        Some(seconds) => (
            Duration::from_secs(seconds),
            Duration::from_secs(rule.max_retry_after_time as u64),
        ),
        None => (
            Duration::from_millis(
                (retries as u64)
                    .saturating_mul(retries as u64)
                    .saturating_mul(100),
            ),
            Duration::from_secs(rule.max_retry_time as u64),
        ),
    };
    if wait <= max { wait } else { Duration::ZERO }
}

/// `404 Not Found` の形の状態の文面
fn status_text(status: StatusCode) -> String {
    match status.canonical_reason() {
        Some(reason) => format!("{} {reason}", status.as_u16()),
        None => status.as_u16().to_string(),
    }
}
