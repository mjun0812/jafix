use std::cell::RefCell;
use std::ops::Range;
use std::sync::LazyLock;

use lindera::dictionary::{DictionaryKind, load_embedded_dictionary};
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::worker::SegmentWorker;

/// 形態素解析の1トークン。品詞などの項目はIPADICの表記で、当てはまらない項目と、未知語の読み・発音は `*` である
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    /// 解析した文字列の中のバイト範囲
    pub range: Range<usize>,
    pub surface: &'a str,
    pub pos: &'static str,
    pub pos_detail_1: &'static str,
    pub pos_detail_2: &'static str,
    pub pos_detail_3: &'static str,
    /// 活用型 (`一段`、`特殊・ダ` など)
    pub conjugated_type: &'static str,
    /// 活用形 (`未然形`、`基本形` など)
    pub conjugated_form: &'static str,
    pub basic_form: &'static str,
    pub reading: &'static str,
    pub pronunciation: &'static str,
}

static SEGMENTER: LazyLock<Segmenter> = LazyLock::new(|| {
    let dictionary =
        load_embedded_dictionary(DictionaryKind::IPADIC).expect("IPADICはbinaryに埋め込んである");
    // 空白のトークンは、語の連続を断つのに使うので残す
    Segmenter::new(Mode::Normal, dictionary, None).keep_whitespace(true)
});

thread_local! {
    // 文や本文の範囲を解析するたびに、linderaの作業領域を同じスレッドで使い回す。
    static WORKER: RefCell<SegmentWorker> = RefCell::new(SEGMENTER.new_worker());
}

/// `text` をトークンに分ける。linderaは `、`・`。`・改行・タブで区切り、区切りごとに解析する
pub fn tokenize(text: &str) -> Vec<Token<'_>> {
    let dictionary = &SEGMENTER.dictionary;
    WORKER.with_borrow_mut(|worker| {
        worker
            .segment(text)
            .expect("埋め込みの辞書での解析は失敗しない")
            .into_iter()
            .map(|token| {
                let id = token.word_id.id() as usize;
                let mut details = if token.word_id.is_unknown() {
                    dictionary.unknown_word_details_iter(id)
                } else {
                    dictionary.word_details_iter(id)
                };
                let mut detail = || details.next().unwrap_or("*");
                Token {
                    range: token.byte_start..token.byte_end,
                    surface: &text[token.byte_start..token.byte_end],
                    pos: detail(),
                    pos_detail_1: detail(),
                    pos_detail_2: detail(),
                    pos_detail_3: detail(),
                    // linderaのschemaは5番目を `conjugation_form` と呼ぶが、値は活用型である
                    conjugated_type: detail(),
                    conjugated_form: detail(),
                    basic_form: detail(),
                    reading: detail(),
                    pronunciation: detail(),
                }
            })
            .collect()
    })
}
