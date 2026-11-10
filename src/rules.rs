use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleId {
    FullwidthParen,
    SpaceBetweenJaAndAlnum,
    SpaceBetweenJaAndParen,
}

impl RuleId {
    /// `check` の出力の `[<rule-id>]` に使う名前
    pub fn name(self) -> &'static str {
        todo!("skeleton: not implemented")
    }

    pub fn message(self) -> &'static str {
        todo!("skeleton: not implemented")
    }
}

/// 修正ルールに当たった1箇所。`range` は元のソース上のバイト範囲で、`replacement` に置き換える
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub range: Range<usize>,
    pub replacement: &'static str,
    pub rule: RuleId,
}

/// 本文の範囲の中で修正ルールに当たる箇所を、位置の順に返す
pub fn find(src: &str, prose: &[Range<usize>]) -> Vec<Finding> {
    let _ = (src, prose);
    todo!("skeleton: not implemented")
}
