use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::Path;

use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use clap::ValueEnum;
use encoding_rs::{Encoding, SHIFT_JIS, UTF_8};

#[derive(Clone, Copy, ValueEnum)]
pub enum InputEncoding {
    Auto,
    #[value(name = "utf-8", alias = "utf8")]
    Utf8,
    #[value(name = "shift-jis", alias = "cp932", alias = "windows-31j")]
    ShiftJis,
}

pub struct TextFile {
    pub content: String,
    shift_jis: bool,
}

impl TextFile {
    pub fn read(path: &Path, encoding: InputEncoding) -> io::Result<Self> {
        Self::decode(fs::read(path)?, encoding)
    }

    pub fn decode(bytes: Vec<u8>, encoding: InputEncoding) -> io::Result<Self> {
        if matches!(encoding, InputEncoding::Utf8) {
            return Ok(Self {
                content: String::from_utf8(bytes)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
                shift_jis: false,
            });
        }
        if matches!(encoding, InputEncoding::ShiftJis) {
            return Self::read_shift_jis(&bytes, false);
        }

        let unsupported = "UTF-8またはShift_JISとして判定できない。文字コードが分かる場合は --encoding で指定する";
        // BOMのないUTF-16やISO-2022-JPには、UTF-8としても読める入力がある。
        if bytes.contains(&0)
            || Encoding::for_bom(&bytes).is_some_and(|(encoding, _)| encoding != UTF_8)
            || (bytes.contains(&0x1B)
                && bytes
                    .windows(3)
                    .any(|part| matches!(part, b"\x1B$B" | b"\x1B$@" | b"\x1B$(")))
        {
            return Err(invalid_data(unsupported));
        }
        let bytes = match String::from_utf8(bytes) {
            Ok(content) => {
                return Ok(Self {
                    content,
                    shift_jis: false,
                });
            }
            Err(error) => error.into_bytes(),
        };
        let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
        detector.feed(&bytes, true);
        if detector.guess(None, Utf8Detection::Deny) != SHIFT_JIS {
            return Err(invalid_data(unsupported));
        }
        Self::read_shift_jis(&bytes, true)
    }

    fn read_shift_jis(bytes: &[u8], automatic: bool) -> io::Result<Self> {
        let content = SHIFT_JIS
            .decode_without_bom_handling_and_without_replacement(bytes)
            .ok_or_else(|| invalid_data("Shift_JISとして読み取れない"))?;
        if automatic {
            let (encoded, _, errors) = SHIFT_JIS.encode(&content);
            if errors || encoded != bytes {
                return Err(invalid_data(
                    "Shift_JISで元のバイト列を再現できない。Shift_JISと分かっている場合は --encoding shift-jis で指定する",
                ));
            }
        }
        Ok(Self {
            content: content.into_owned(),
            shift_jis: true,
        })
    }

    pub fn encode<'a>(&self, content: &'a str) -> io::Result<Cow<'a, [u8]>> {
        if !self.shift_jis {
            return Ok(Cow::Borrowed(content.as_bytes()));
        }
        let (encoded, _, errors) = SHIFT_JIS.encode(content);
        // 円記号など、エンコードに成功しても読み直すと文字が変わる場合がある。
        if errors
            || SHIFT_JIS
                .decode_without_bom_handling_and_without_replacement(&encoded)
                .as_deref()
                != Some(content)
        {
            return Err(invalid_data("修正後の内容をShift_JISで保存できない"));
        }
        Ok(encoded)
    }
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
