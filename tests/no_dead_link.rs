mod common;

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use common::{Sandbox, stdout};
use serde_json::json;

/// 127.0.0.1の空いたportで要求を受け、宛先ごとに決めた応答を返し、受けた要求の方式と宛先を記録するHTTPサーバー。
/// `/ok` と `/skip` は200、`/moved` は `/ok` への301、`/moved-to-fragment` は `/ok#loc` への301、`/head-not-allowed` は
/// `HEAD` に405で `GET` に200、それ以外は404を返す
struct Server {
    port: u16,
    requests: Arc<Mutex<Vec<(String, String)>>>,
}

impl Server {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = Arc::default();
        let log = Arc::clone(&requests);
        // 接続を待つスレッドは、テストのプロセスが終わるときに一緒に終わる
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                respond(stream, &log);
            }
        });
        Self { port, requests }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    fn requests(&self) -> Vec<(String, String)> {
        self.requests.lock().unwrap().clone()
    }
}

fn respond(mut stream: TcpStream, log: &Mutex<Vec<(String, String)>>) {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let (Some(method), Some(path)) = (parts.next(), parts.next()) else {
        return;
    };
    let (method, path) = (method.to_owned(), path.to_owned());
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
            break;
        }
    }
    log.lock().unwrap().push((method.clone(), path.clone()));
    let (status, location) = match (method.as_str(), path.as_str()) {
        (_, "/ok" | "/skip") => ("200 OK", ""),
        (_, "/moved") => ("301 Moved Permanently", "Location: /ok\r\n"),
        (_, "/moved-to-fragment") => ("301 Moved Permanently", "Location: /ok#loc\r\n"),
        ("HEAD", "/head-not-allowed") => ("405 Method Not Allowed", ""),
        (_, "/head-not-allowed") => ("200 OK", ""),
        _ => ("404 Not Found", ""),
    };
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\n{location}Content-Length: 0\r\nConnection: close\r\n\r\n"
    );
}

fn request(method: &str, path: &str) -> (String, String) {
    (method.to_owned(), path.to_owned())
}

/// `jafix check` の標準出力のうち、`no-dead-link` の指摘の行
fn dead_link_lines(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|line| line.ends_with(" [no-dead-link]"))
        .collect()
}

#[test]
fn check_reports_dead_and_redirected_urls() {
    let server = Server::start();
    let sandbox = Sandbox::new();
    sandbox.write(
        "t.md",
        &format!(
            "[a]({})\n\n[b]({})\n\n[c]({})\n\n[d]({})\n",
            server.url("/dead"),
            server.url("/ok"),
            server.url("/moved"),
            server.url("/head-not-allowed"),
        ),
    );
    sandbox.write_xdg_config("[builtin]\nno-dead-link = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(output.status.code(), Some(1));
    let ok = server.url("/ok");
    assert_eq!(
        dead_link_lines(stdout(&output)),
        [
            format!(
                "t.md:1:5: リンク先に届かない: {} (404 Not Found) [no-dead-link]",
                server.url("/dead")
            ),
            format!(
                "t.md:5:5: リンク先が {ok} へリダイレクトする (301 Moved Permanently)。宛先を {ok} にする [no-dead-link]"
            ),
        ]
    );
}

#[test]
fn only_check_requests_external_urls() {
    let server = Server::start();
    let sandbox = Sandbox::new();
    let content = format!(
        "[a]({})\n\n[b]({})\n\n[c](missing.md)\n",
        server.url("/dead"),
        server.url("/moved"),
    );
    let path = sandbox.write("t.md", &content);
    sandbox.write_xdg_config("[builtin]\nno-dead-link = true\n");
    let claude = json!({
        "hook_event_name": "PostToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": path, "content": content},
        "tool_response": {
            "type": "create",
            "filePath": path,
            "content": content,
            "structuredPatch": [],
            "originalFile": null,
            "userModified": false,
        },
    })
    .to_string();
    let added: String = content.lines().map(|line| format!("+{line}\n")).collect();
    let codex = json!({
        "hook_event_name": "PostToolUse",
        "cwd": sandbox.path(""),
        "tool_name": "apply_patch",
        "tool_input": {"command": format!("*** Begin Patch\n*** Add File: t.md\n{added}*** End Patch\n")},
        "tool_response": "Success. Updated the following files:\nA t.md\n",
    })
    .to_string();
    for (agent, input) in [("claude", claude), ("codex", codex)] {
        let output = sandbox.hook(agent, &input);
        // ローカルのファイルはhookでも確かめるので、外部のURLの指摘だけが出ない
        assert_eq!(output.status.code(), Some(2), "{agent}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            dead_link_lines(&stderr),
            [format!(
                "{}:5:5: リンク先のファイルが無い: missing.md [no-dead-link]",
                path.display()
            )],
            "{agent}"
        );
    }
    let output = sandbox.run(&["fix", "t.md"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(server.requests(), []);

    // 同じ設定と入力で、`jafix check` はサーバーに要求する
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(dead_link_lines(stdout(&output)).len(), 3);
    assert!(!server.requests().is_empty());
}

#[test]
fn retry_and_prefer_get_change_requests() {
    let server = Server::start();
    let sandbox = Sandbox::new();
    sandbox.write("t.md", &format!("[a]({})\n", server.url("/dead")));
    sandbox.write_xdg_config(&format!(
        "[builtin]\nno-dead-link = {{ retry = 1, preferGET = [\"{}\"] }}\n",
        server.url("")
    ));
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(dead_link_lines(stdout(&output)).len(), 1);
    assert_eq!(
        server.requests(),
        [request("GET", "/dead"), request("GET", "/dead")]
    );
}

#[test]
fn check_requests_each_url_once() {
    let server = Server::start();
    let sandbox = Sandbox::new();
    for name in ["a.md", "b.md"] {
        sandbox.write(name, &format!("[a]({})\n", server.url("/dead")));
    }
    sandbox.write_xdg_config("[builtin]\nno-dead-link = { retry = 1 }\n");
    let output = sandbox.run(&["check", "a.md", "b.md"]);
    let found = dead_link_lines(stdout(&output));
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].starts_with("a.md:1:5: "), "{found:?}");
    assert!(found[1].starts_with("b.md:1:5: "), "{found:?}");
    assert_eq!(
        server.requests(),
        [request("HEAD", "/dead"), request("GET", "/dead")]
    );
}

#[test]
fn bare_url_ends_before_unmatched_fullwidth_paren() {
    let server = Server::start();
    let sandbox = Sandbox::new();
    sandbox.write(
        "t.md",
        &format!(
            "公式（{}）を参照。\n\n別の（{}）も見る。\n",
            server.url("/ok"),
            server.url("/skip"),
        ),
    );
    sandbox.write_xdg_config("[builtin]\nno-dead-link = { ignore = [\"**/skip\"] }\n");
    let output = sandbox.run(&["check", "t.md"]);
    assert_eq!(dead_link_lines(stdout(&output)), Vec::<&str>::new());
    assert_eq!(server.requests(), [request("HEAD", "/ok")]);
}

#[test]
fn definition_label_with_a_backtick_reports_at_the_destination() {
    for (content, expected) in [
        (
            "[a`b]: /x`\n\nfoo bar baz\n",
            "t.md:1:8: リンク先のファイルが無い: /x` [no-dead-link]",
        ),
        (
            "[`a]: /y`\n",
            "t.md:1:7: リンク先のファイルが無い: /y` [no-dead-link]",
        ),
    ] {
        let sandbox = Sandbox::new();
        sandbox.write("t.md", content);
        sandbox.write_xdg_config("[builtin]\nno-dead-link = true\n");
        let output = sandbox.run(&["check", "t.md"]);
        assert_eq!(output.status.code(), Some(1), "{content:?}");
        assert_eq!(dead_link_lines(stdout(&output)), [expected], "{content:?}");
    }
}

#[test]
fn urls_differing_only_in_fragment_are_requested_once() {
    let server = Server::start();
    let sandbox = Sandbox::new();
    let moved = server.url("/moved");
    sandbox.write("t.md", &format!("[a]({moved}#sec)\n\n[b]({moved})\n"));
    sandbox.write_xdg_config("[builtin]\nno-dead-link = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let ok = server.url("/ok");
    // `#` から後ろは指摘の文面のリダイレクト先に残る
    assert_eq!(
        dead_link_lines(stdout(&output)),
        [
            format!(
                "t.md:1:5: リンク先が {ok}#sec へリダイレクトする (301 Moved Permanently)。宛先を {ok}#sec にする [no-dead-link]"
            ),
            format!(
                "t.md:3:5: リンク先が {ok} へリダイレクトする (301 Moved Permanently)。宛先を {ok} にする [no-dead-link]"
            ),
        ]
    );
    assert_eq!(
        server.requests(),
        [request("HEAD", "/moved"), request("HEAD", "/ok")]
    );
}

#[test]
fn location_fragment_wins_over_the_original_fragment() {
    let server = Server::start();
    let sandbox = Sandbox::new();
    sandbox.write(
        "t.md",
        &format!("[a]({}#sec)\n", server.url("/moved-to-fragment")),
    );
    sandbox.write_xdg_config("[builtin]\nno-dead-link = true\n");
    let output = sandbox.run(&["check", "t.md"]);
    let to = server.url("/ok#loc");
    assert_eq!(
        dead_link_lines(stdout(&output)),
        [format!(
            "t.md:1:5: リンク先が {to} へリダイレクトする (301 Moved Permanently)。宛先を {to} にする [no-dead-link]"
        )]
    );
}
