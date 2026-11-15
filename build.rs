//! `@textlint-ja/no-synonyms` の同義語辞書 (`src/rules/morph/no_synonyms/sudachi-synonyms.tsv`) から、見出しで引く表を
//! `OUT_DIR` に作る。hookは1回ごとに別のプロセスなので、実行時に表を作らずに済むよう、build時に作って埋め込む。
//!
//! - `synonyms-headwords.tsv`: 1行が1つの見出しで、`見出し\tグループの番号,語彙番号,英字の印;…`。グループの番号は
//!   辞書のデータの行の番号 (0始まり、`#` の行を除く) で、語彙番号が無ければ空である。行は見出しのバイト列の順に並ぶ
//! - `synonyms-table.bin`: 見出しのFNV-1a (32bit) の下位ビットで位置を決めた開番地法の表。`TABLE_SIZE` 個のリトル
//!   エンディアンの `u32` で、値は `synonyms-headwords.tsv` の行の先頭のバイト位置に1を足したもの、空きは0である

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

const DICTIONARY: &str = "src/rules/morph/no_synonyms/sudachi-synonyms.tsv";

/// 表の大きさ。見出し (68,701個) の2倍に近い2の冪にする
const TABLE_SIZE: usize = 1 << 17;

fn main() {
    println!("cargo::rerun-if-changed={DICTIONARY}");
    let dictionary = std::fs::read_to_string(DICTIONARY).expect("同義語辞書を読めない");
    let mut headwords: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (group, line) in dictionary
        .lines()
        .filter(|line| !line.starts_with('#'))
        .enumerate()
    {
        for item in line.split('\t') {
            let mut fields = item.splitn(3, ',');
            let (Some(number), Some(alphabet), Some(headword)) =
                (fields.next(), fields.next(), fields.next())
            else {
                panic!(
                    "同義語辞書の{}行目の項目の形が誤っている: {item}",
                    group + 1
                );
            };
            headwords
                .entry(headword)
                .or_default()
                .push(format!("{group},{number},{alphabet}"));
        }
    }
    assert!(headwords.len() * 3 / 2 < TABLE_SIZE, "表が小さすぎる");
    let mut lines = String::new();
    let mut table = vec![0u32; TABLE_SIZE];
    for (headword, values) in &headwords {
        let mut slot = fnv1a(headword) as usize & (TABLE_SIZE - 1);
        while table[slot] != 0 {
            slot = (slot + 1) & (TABLE_SIZE - 1);
        }
        table[slot] = u32::try_from(lines.len() + 1).expect("見出しの表は4GB未満である");
        writeln!(lines, "{headword}\t{}", values.join(";")).unwrap();
    }
    let out = std::env::var_os("OUT_DIR").expect("cargoがOUT_DIRを渡す");
    let out = Path::new(&out);
    std::fs::write(out.join("synonyms-headwords.tsv"), lines).unwrap();
    let bytes: Vec<u8> = table.iter().flat_map(|slot| slot.to_le_bytes()).collect();
    std::fs::write(out.join("synonyms-table.bin"), bytes).unwrap();
}

/// 32bitのFNV-1aの初期値
const FNV_OFFSET_BASIS: u32 = 0x811c_9dc5;

/// 32bitのFNV-1aの乗数
const FNV_PRIME: u32 = 0x0100_0193;

/// 32bitのFNV-1a。実行時の照合は、トークンの表層を1つずつ足して同じ値を求める
fn fnv1a(text: &str) -> u32 {
    text.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(FNV_PRIME)
    })
}
