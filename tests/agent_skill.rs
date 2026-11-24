use std::fs;
use std::path::Path;

const SKILL_DIR: &str = "skills/jafix";

/// Agent Skills仕様がfrontmatterに認めるキー
const ALLOWED_KEYS: [&str; 6] = [
    "name",
    "description",
    "license",
    "compatibility",
    "metadata",
    "allowed-tools",
];

struct Skill {
    fields: Vec<(String, String)>,
    body_lines: usize,
}

impl Skill {
    /// frontmatterの値はどれも1行の文字列なので、YAMLとしては読まず、行頭の `<キー>:` で区切る。
    /// 行は `lines` で分け、Windowsのcheckoutで改行がCRLFになっても同じに読む
    fn read() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(SKILL_DIR)
            .join("SKILL.md");
        let content = fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(
            lines.first(),
            Some(&"---"),
            "SKILL.mdが `---` の行で始まらない"
        );
        let end = 1 + lines[1..]
            .iter()
            .position(|line| *line == "---")
            .expect("frontmatterを閉じる `---` の行が無い");
        let fields = lines[1..end]
            .iter()
            .filter(|line| !line.starts_with([' ', '\t']))
            .map(|line| {
                let (key, value) = line
                    .split_once(':')
                    .unwrap_or_else(|| panic!("キーの無い行: {line}"));
                (key.to_owned(), value.trim().to_owned())
            })
            .collect();
        Self {
            fields,
            body_lines: lines.len() - end - 1,
        }
    }

    fn field(&self, key: &str) -> &str {
        self.fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or_else(|| panic!("frontmatterに `{key}` が無い"))
    }
}

#[test]
fn skill_frontmatter_uses_only_keys_defined_by_the_agent_skills_spec() {
    let skill = Skill::read();
    for (key, _) in &skill.fields {
        assert!(
            ALLOWED_KEYS.contains(&key.as_str()),
            "仕様に無いキー: {key}"
        );
    }
}

#[test]
fn skill_name_matches_its_directory_and_the_allowed_characters() {
    let skill = Skill::read();
    let name = skill.field("name");
    let directory = Path::new(SKILL_DIR).file_name().unwrap();
    assert_eq!(name, directory.to_str().unwrap());
    assert!((1..=64).contains(&name.chars().count()), "{name}");
    assert!(
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
        "{name}"
    );
    assert!(!name.starts_with('-') && !name.ends_with('-'), "{name}");
    assert!(!name.contains("--"), "{name}");
}

#[test]
fn skill_description_is_between_1_and_1024_characters() {
    let skill = Skill::read();
    let length = skill.field("description").chars().count();
    assert!((1..=1024).contains(&length), "{length}文字");
}

#[test]
fn skill_body_is_shorter_than_500_lines() {
    let skill = Skill::read();
    assert!(skill.body_lines < 500, "{}行", skill.body_lines);
}
