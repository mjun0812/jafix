use crate::sentence::Sentence;

/// 文の文字列と、元のソースへの位置の対応。文の文字列は、文の子の `Inline` から作る。
/// 段落の直下の文字列は元のソースのまま、インラインコードは中身と同じ文字数の `ー`、画像はalt、
/// リンクや強調は中の文字列で、HTMLのタグ・ハード改行・脚注への参照は入らない
pub struct Projection {
    pub text: String,
    // skeleton: not implemented (位置の対応を持つ項目を足す)
}

impl Projection {
    pub fn new(sentence: &Sentence) -> Self {
        let _ = sentence;
        todo!("skeleton: not implemented")
    }

    /// `text` の中のバイト位置を、元のソースのバイト位置に戻す。インラインコードと画像の中の位置は、その要素の先頭に戻る
    pub fn source(&self, index: usize) -> usize {
        let _ = index;
        todo!("skeleton: not implemented")
    }
}
