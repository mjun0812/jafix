//! prh 6.1.0の辞書の読み込みと照合。文字列の中の置き換えの箇所を返すだけで、どの本文の範囲に当てるかと、
//! 箇所を修正にするか指摘にするかは呼び出す側が決める

use std::ops::Range;
use std::path::{Path, PathBuf};

use regress::Regex;
use serde_json::{Map, Value};

use crate::js_regex::{self, Flags};
use crate::rules::drop_overlaps;

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
    /// まとめた後のルール。出どころ (`imports` の辞書を含む) をまたいで `expected` が同じルールは、先にある出どころの
    /// ものだけを持つ。1つの出どころの中で `expected` が同じルールは、どれも持つ
    rules: Vec<Rule>,
}

/// 辞書の1つのルール
#[derive(Debug)]
pub struct Rule {
    /// 置き換え後の文字列。`$1`〜`$99` で捕捉グループを参照する
    pub expected: String,
    /// ルールの `prh` に書いた補足。前後の空白を除き、空白の続きを半角空白1つにしたもの
    pub note: Option<String>,
    /// ルールを読んだ出どころの、`Dictionary::load` に渡した一覧での番号。`imports` で取り込んだルールは、取り込んだ側の番号である
    pub source: usize,
    regex: Regex,
    literal_prefix: Option<String>,
    /// 正規表現の捕捉グループの数。`expected` の `$` が参照かを決めるのに使うので、`expected` が `$` を含まなければ0にする
    groups: usize,
    /// `regexpMustEmpty` のグループの番号
    must_empty: Option<usize>,
    /// `ignoreRules` の `pattern` と比べる、JavaScriptの `RegExp.prototype.toString()` の形
    pattern_string: String,
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
        let mut rules = Vec::new();
        for (index, source) in sources.iter().enumerate() {
            let loaded = match source {
                Source::Path(path) => load_file(path, false, index, &mut Vec::new()),
                Source::Contents { yaml, base } => {
                    load_yaml(yaml, base, false, index, &mut Vec::new())
                }
            }
            .map_err(|reason| LoadError {
                source: index,
                reason,
            })?;
            merge(&mut rules, loaded);
        }
        Ok(Self { rules })
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// `text` の中の置き換えの箇所を、重なりを解いて位置の順に返す
    pub fn replacements(&self, text: &str) -> Vec<Replacement> {
        let mut replacements = Vec::new();
        for (index, rule) in self.rules.iter().enumerate() {
            rule.push_replacements(text, index, &mut replacements);
        }
        drop_overlaps(&mut replacements, |replacement| &replacement.range);
        replacements
    }
}

/// `incoming` のうち、`rules` に `expected` が同じルールが無いものを足す (prhの `merge`)。`incoming` の中で
/// `expected` が重なるルールは、どれも残す
fn merge(rules: &mut Vec<Rule>, incoming: Vec<Rule>) {
    let kept = rules.len();
    for rule in incoming {
        if !rules[..kept]
            .iter()
            .any(|kept| kept.expected == rule.expected)
        {
            rules.push(rule);
        }
    }
}

/// `path` の辞書を読む。`loading` は読み込み中の辞書のファイルで、`imports` の循環を見つけるのに使う
fn load_file(
    path: &Path,
    disable_imports: bool,
    source: usize,
    loading: &mut Vec<PathBuf>,
) -> Result<Vec<Rule>, String> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if loading.contains(&canonical) {
        return Err("`imports` が循環している".to_owned());
    }
    let yaml = std::fs::read_to_string(path).map_err(|e| format!("読めない: {e}"))?;
    let base = path.parent().unwrap_or(Path::new(""));
    loading.push(canonical);
    let rules = load_yaml(&yaml, base, disable_imports, source, loading);
    loading.pop();
    rules
}

/// YAMLの辞書を読み、自身のルールの後に `imports` の辞書のルールをまとめる。`base` は `imports` の相対パスを解決するディレクトリ
fn load_yaml(
    yaml: &str,
    base: &Path,
    disable_imports: bool,
    source: usize,
    loading: &mut Vec<PathBuf>,
) -> Result<Vec<Rule>, String> {
    let document = parse_yaml(yaml)?;
    let table = match &document {
        Value::Null => return Ok(Vec::new()),
        Value::Object(table) => table,
        _ => return Err("辞書は表で書く".to_owned()),
    };
    let mut rules = match table.get("rules") {
        None => Vec::new(),
        Some(Value::Array(values)) => values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                rule(value, source).map_err(|reason| format!("`rules` の{}番目: {reason}", i + 1))
            })
            .collect::<Result<_, _>>()?,
        Some(_) => return Err("`rules` は配列で書く".to_owned()),
    };
    if disable_imports {
        return Ok(rules);
    }
    for import in imports(table.get("imports"))? {
        let path = base.join(&import.path);
        let mut imported = load_file(&path, import.disable_imports, source, loading)
            .map_err(|reason| format!("`imports` の `{}`: {reason}", path.display()))?;
        imported.retain(|rule| {
            !import
                .ignore_rules
                .iter()
                .any(|ignore| ignore.matches(rule))
        });
        merge(&mut rules, imported);
    }
    Ok(rules)
}

/// prhとtextlintのルールが使う `js-yaml` 4と同じく、`yes`・`no`・`on`・`off` を文字列にしてYAMLを読む
pub fn parse_yaml(yaml: &str) -> Result<Value, String> {
    let options = serde_saphyr::options! {
        strict_booleans: true,
        with_snippet: false,
    };
    serde_saphyr::from_str_with_options(yaml, options)
        .map_err(|e| format!("YAMLとして読めない: {e}"))
}

/// `imports` の1つの要素
struct Import {
    path: String,
    disable_imports: bool,
    ignore_rules: Vec<IgnoreRule>,
}

/// `ignoreRules` の1つの要素。`expected` か `pattern` が等しいルールを除く
struct IgnoreRule {
    pattern: Option<String>,
    expected: Option<String>,
}

impl IgnoreRule {
    fn matches(&self, rule: &Rule) -> bool {
        self.expected.as_ref() == Some(&rule.expected)
            || self.pattern.as_ref() == Some(&rule.pattern_string)
    }
}

const IMPORTS_TYPE: &str = "`imports` は文字列か、要素が文字列か表の配列で書く";

fn imports(value: Option<&Value>) -> Result<Vec<Import>, String> {
    let values = match value {
        None => return Ok(Vec::new()),
        Some(Value::String(path)) => return Ok(vec![import_path(path)]),
        Some(Value::Array(values)) => values,
        Some(_) => return Err(IMPORTS_TYPE.to_owned()),
    };
    values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            import(value).map_err(|reason| format!("`imports` の{}番目: {reason}", i + 1))
        })
        .collect()
}

fn import_path(path: &str) -> Import {
    Import {
        path: path.to_owned(),
        disable_imports: false,
        ignore_rules: Vec::new(),
    }
}

fn import(value: &Value) -> Result<Import, String> {
    let table = match value {
        Value::String(path) => return Ok(import_path(path)),
        Value::Object(table) => table,
        _ => return Err("文字列か表で書く".to_owned()),
    };
    let path = optional_string(table, "path")?.ok_or("`path` が無い")?;
    let disable_imports = match table.get("disableImports") {
        None => false,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err("`disableImports` は `true` か `false` で書く".to_owned()),
    };
    let ignore_rules = match table.get("ignoreRules") {
        None => Vec::new(),
        Some(Value::Array(values)) => values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                ignore_rule(value)
                    .map_err(|reason| format!("`ignoreRules` の{}番目: {reason}", i + 1))
            })
            .collect::<Result<_, _>>()?,
        Some(_) => return Err("`ignoreRules` は配列で書く".to_owned()),
    };
    Ok(Import {
        path,
        disable_imports,
        ignore_rules,
    })
}

fn ignore_rule(value: &Value) -> Result<IgnoreRule, String> {
    match value {
        Value::String(pattern) => Ok(IgnoreRule {
            pattern: Some(pattern.clone()),
            expected: None,
        }),
        Value::Object(table) => Ok(IgnoreRule {
            pattern: optional_string(table, "pattern")?,
            expected: optional_string(table, "expected")?,
        }),
        _ => Err("文字列か表で書く".to_owned()),
    }
}

fn optional_string(table: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match table.get(key) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(format!("`{key}` は文字列で書く")),
    }
}

/// `rules` の1つの要素を読み、正規表現を作って `specs` を確かめる
fn rule(value: &Value, source: usize) -> Result<Rule, String> {
    let empty = Map::new();
    let (expected, table) = match value {
        Value::String(expected) => (expected.clone(), &empty),
        Value::Object(table) => (
            optional_string(table, "expected")?.ok_or("`expected` が無い")?,
            table,
        ),
        _ => return Err("文字列か表で書く".to_owned()),
    };
    let word_boundary = match table.get("options") {
        None => false,
        Some(Value::Object(options)) => match options.get("wordBoundary") {
            None => false,
            Some(Value::Bool(value)) => *value,
            Some(_) => {
                return Err("`options` の `wordBoundary` は `true` か `false` で書く".to_owned());
            }
        },
        Some(_) => return Err("`options` は表で書く".to_owned()),
    };
    let (compiled, shown, flags) = match ["pattern", "patterns"]
        .into_iter()
        .find_map(|key| Some((key, table.get(key)?)))
    {
        Some((key, value)) => {
            patterns_source(value, word_boundary).map_err(|reason| format!("`{key}`: {reason}"))?
        }
        None => {
            let (compiled, shown) = bounded(&spread_alphanumerics(&expected), word_boundary);
            (
                compiled,
                shown,
                Flags {
                    multiline: true,
                    ..Flags::default()
                },
            )
        }
    };
    let regex = js_regex::compile(&compiled, flags)
        .map_err(|reason| format!("`expected` の「{expected}」の `pattern` を{reason}"))?;
    let groups = if expected.contains('$') {
        js_regex::group_count(&compiled, flags)
    } else {
        0
    };
    let must_empty = match table.get("regexpMustEmpty") {
        None => None,
        Some(Value::String(value)) => Some(
            value
                .strip_prefix('$')
                .filter(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|digits| digits.parse().ok())
                .ok_or("`regexpMustEmpty` は `$1` のように `$` と数字で書く")?,
        ),
        Some(_) => return Err("`regexpMustEmpty` は `$1` のように `$` と数字で書く".to_owned()),
    };
    let note = match table.get("prh") {
        None => None,
        Some(Value::String(note)) => Some(note.split_whitespace().collect::<Vec<_>>().join(" ")),
        Some(_) => return Err("`prh` は文字列で書く".to_owned()),
    };
    let rule = Rule {
        pattern_string: pattern_string(&shown, flags),
        literal_prefix: literal_prefix(&compiled, flags),
        expected,
        note,
        source,
        regex,
        groups,
        must_empty,
    };
    rule.check_specs(table.get("specs"))?;
    Ok(rule)
}

/// 正規表現の先頭に必ず現れる文字列。入力に無いときに照合を省くために使う
fn literal_prefix(source: &str, flags: Flags) -> Option<String> {
    if flags.ignore_case {
        return None;
    }

    let prefix_source = source.strip_prefix('^').unwrap_or(source);
    let mut prefix_end = prefix_source
        .char_indices()
        .take_while(|(_, c)| {
            !matches!(
                c,
                '^' | '$' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
            )
        })
        .last()
        .map_or(0, |(index, c)| index + c.len_utf8());
    if prefix_source[prefix_end..]
        .chars()
        .next()
        .is_some_and(|c| matches!(c, '?' | '*' | '{'))
    {
        prefix_end = prefix_source[..prefix_end]
            .char_indices()
            .last()
            .map_or(0, |(index, _)| index);
    }
    if prefix_source[..prefix_end].chars().count() < 2 {
        return None;
    }

    let mut groups = 0;
    let mut escaped = false;
    let mut in_class = false;
    for c in source.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            '[' if !in_class => in_class = true,
            ']' if in_class => in_class = false,
            _ if in_class => {}
            '(' => groups += 1,
            ')' => groups -= 1,
            '|' if groups == 0 => return None,
            _ => {}
        }
    }

    Some(prefix_source[..prefix_end].to_owned())
}

/// `pattern`・`patterns` の値から、照合に使う正規表現の元の文字列、`ignoreRules` と比べる元の文字列、フラグを作る
fn patterns_source(value: &Value, word_boundary: bool) -> Result<(String, String, Flags), String> {
    let type_error = || "空でない文字列か、空でない文字列の配列で書く".to_owned();
    let values: Vec<&str> = match value {
        Value::String(value) => vec![value],
        Value::Array(values) => values
            .iter()
            .map(Value::as_str)
            .collect::<Option<_>>()
            .ok_or_else(type_error)?,
        _ => return Err(type_error()),
    };
    if values.iter().any(|value| value.is_empty()) {
        return Err(type_error());
    }
    let parts: Vec<_> = values.iter().map(|value| js_regex::prh(value)).collect();
    if let [(source, flags)] = parts.as_slice()
        && value.is_string()
    {
        let (compiled, shown) = bounded(source, word_boundary);
        return Ok((compiled, shown, *flags));
    }
    let flags = parts.first().map(|(_, flags)| *flags).unwrap_or(Flags {
        multiline: true,
        ..Flags::default()
    });
    if parts.iter().any(|(_, other)| *other != flags) {
        return Err("要素のフラグ (`i`・`s`) が食い違う。どの要素にも同じフラグを書く".to_owned());
    }
    let (compiled, shown): (Vec<_>, Vec<_>) = parts
        .iter()
        .map(|(source, _)| bounded(source, word_boundary))
        .unzip();
    Ok((
        format!("(?:{})", compiled.join("|")),
        format!("(?:{})", shown.join("|")),
        flags,
    ))
}

/// `wordBoundary` なら `source` を `\b` で挟んだ、照合に使う元の文字列と、`ignoreRules` と比べる元の文字列を返す。
/// prhは括弧で囲まずにつなぐので、`/a|b/` の `\b` が片方ずつにしか掛からないが、照合では全体を挟む
fn bounded(source: &str, word_boundary: bool) -> (String, String) {
    if word_boundary {
        (format!(r"\b(?:{source})\b"), format!(r"\b{source}\b"))
    } else {
        (source.to_owned(), source.to_owned())
    }
}

/// `expected` だけのルールの正規表現の元の文字列。ASCIIの英字は大文字・小文字・全角の大文字・全角の小文字の文字クラス、
/// ASCIIの数字は半角と全角の文字クラスにし、ほかの文字は正規表現の記号をエスケープする
fn spread_alphanumerics(expected: &str) -> String {
    let fullwidth = |c: char| {
        char::from_u32(c as u32 - 0x21 + 0xFF01).expect("ASCIIの英数字には全角の文字がある")
    };
    let mut source = String::new();
    for c in expected.chars() {
        if c.is_ascii_alphabetic() {
            let (upper, lower) = (c.to_ascii_uppercase(), c.to_ascii_lowercase());
            source.extend(['[', upper, lower, fullwidth(upper), fullwidth(lower), ']']);
        } else if c.is_ascii_digit() {
            source.extend(['[', c, fullwidth(c), ']']);
        } else {
            source.push_str(&js_regex::escape(c.encode_utf8(&mut [0; 4])));
        }
    }
    source
}

/// JavaScriptの `RegExp.prototype.toString()` の形。文字クラスの外の `/` は `\/`、改行は `\n` にし、フラグは
/// `g`・`i`・`m`・`s`・`u` のうち付くものをこの順に並べる
fn pattern_string(source: &str, flags: Flags) -> String {
    let mut shown = String::from("/");
    let mut escaped = false;
    let mut in_class = false;
    for c in source.chars() {
        match c {
            '\n' => shown.push_str("\\n"),
            '/' if !escaped && !in_class => shown.push_str("\\/"),
            _ => shown.push(c),
        }
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            '[' => in_class = true,
            ']' => in_class = false,
            _ => {}
        }
    }
    shown.push_str("/g");
    for (flag, on) in [('i', flags.ignore_case), ('m', true), ('s', flags.dot_all)] {
        if on {
            shown.push(flag);
        }
    }
    shown.push('u');
    shown
}

impl Rule {
    /// `text` の中の、このルールの置き換えの箇所を `replacements` に足す。`index` は `Dictionary::rules` での番号
    fn push_replacements(&self, text: &str, index: usize, replacements: &mut Vec<Replacement>) {
        if self
            .literal_prefix
            .as_deref()
            .is_some_and(|prefix| !text.contains(prefix))
        {
            return;
        }
        for m in self.regex.find_iter(text) {
            if m.range.is_empty() {
                continue;
            }
            if let Some(group) = self.must_empty
                && m.group(group).is_some_and(|range| !range.is_empty())
            {
                continue;
            }
            let matched = &text[m.range()];
            if matched == self.expected {
                continue;
            }
            let replaced = expand(&self.expected, &m, text);
            if replaced == matched {
                continue;
            }
            replacements.push(Replacement {
                range: m.range(),
                text: replaced,
                rule: index,
            });
        }
    }

    /// `specs` の各要素で、このルールだけを `from` に当てた結果が `to` と同じかを確かめる
    fn check_specs(&self, specs: Option<&Value>) -> Result<(), String> {
        let specs = match specs {
            None => return Ok(()),
            Some(Value::Array(specs)) => specs,
            Some(_) => return Err("`specs` は配列で書く".to_owned()),
        };
        for (i, spec) in specs.iter().enumerate() {
            let field = |key: &str| match spec.get(key) {
                Some(Value::String(value)) if !value.is_empty() => Ok(value.as_str()),
                _ => Err(format!(
                    "`specs` の{}番目: `{key}` は空でない文字列で書く",
                    i + 1
                )),
            };
            let (from, to) = (field("from")?, field("to")?);
            let mut replacements = Vec::new();
            self.push_replacements(from, 0, &mut replacements);
            drop_overlaps(&mut replacements, |replacement| &replacement.range);
            let mut result = String::new();
            let mut copied = 0;
            for Replacement { range, text, .. } in &replacements {
                result.push_str(&from[copied..range.start]);
                result.push_str(text);
                copied = range.end;
            }
            result.push_str(&from[copied..]);
            if result != to {
                return Err(format!(
                    "`expected` の「{}」の `specs` の{}番目: `from` の「{from}」に当てた結果「{result}」が `to` の「{to}」と違う",
                    self.expected,
                    i + 1
                ));
            }
        }
        Ok(())
    }
}

impl Rule {
    /// `expected` のうち、捕捉グループの参照を除いた部分。参照の区切り方は `expand` と同じである
    pub fn fixed_parts(&self) -> Vec<js_regex::FixedPart> {
        js_regex::fixed_parts(
            pieces(&self.expected, self.groups).map(|piece| match piece {
                Piece::Text(text) => Some(text),
                Piece::Group(_) => None,
            }),
        )
    }
}

/// `expected` の `$` と1〜2桁の数字を、一致の捕捉グループに展開する。参加しなかったグループは空文字列にする
fn expand(expected: &str, m: &regress::Match, text: &str) -> String {
    let mut out = String::with_capacity(expected.len());
    for piece in pieces(expected, m.captures.len()) {
        match piece {
            Piece::Text(piece) => out.push_str(piece),
            Piece::Group(number) => {
                if let Some(range) = m.group(number) {
                    out.push_str(&text[range]);
                }
            }
        }
    }
    out
}

/// `expected` の1片
enum Piece<'e> {
    /// そのまま入れる文字列。参照にならない `$` と数字は、書いたまま含む
    Text(&'e str),
    /// 捕捉グループの番号
    Group(usize),
}

/// `expected` を、`$` と1〜2桁の数字の参照で区切る。2桁を先に読み、1から `groups` (捕捉グループの数) までの番号を参照とし、
/// `$0` と `groups` を超える番号は書いたまま残す
fn pieces(expected: &str, groups: usize) -> impl Iterator<Item = Piece<'_>> {
    let mut rest = expected;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        if let Some(after) = rest.strip_prefix('$') {
            let digits = after.bytes().take(2).take_while(u8::is_ascii_digit).count();
            let number: usize = after[..digits].parse().unwrap_or(0);
            if (1..=groups).contains(&number) {
                rest = &after[digits..];
                return Some(Piece::Group(number));
            }
        }
        // 参照にならない `$` は、次の `$` の手前までの文字列に含める
        let skip = usize::from(rest.starts_with('$'));
        let end = rest[skip..].find('$').map_or(rest.len(), |i| i + skip);
        let (text, after) = rest.split_at(end);
        rest = after;
        Some(Piece::Text(text))
    })
}
