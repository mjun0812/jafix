# Configuration

Without a configuration file, jafix runs the [10 default rules](#defaults).
Write TOML to change the rules.

## Configuration files {#files}

| Kind    | Path                          | Use it for                                  |
| ------- | ----------------------------- | ------------------------------------------- |
| Global  | `~/.config/jafix/config.toml` | Settings for everything you write           |
| Project | `.jafix.toml`                 | Settings shared with a team in a repository |

If `XDG_CONFIG_HOME` is set, the global file is `$XDG_CONFIG_HOME/jafix/config.toml`.

The project file is the first `.jafix.toml` found by walking up from the directory of the target file.
The search does not go above the directory that contains `.git`.

## Precedence {#precedence}

jafix applies the defaults, then the global file, then the project file.
A later setting overrides an earlier one for the same item.

| Item                           | Overridden per                                    |
| ------------------------------ | ------------------------------------------------- |
| `[builtin]` and `[filters]`    | Rule. Rules you do not mention keep their setting |
| Options of a rule              | Table. Options you omit return to their defaults  |
| `exclude` and `report-exclude` | List                                              |

In this example, the `sentence-length` limit becomes 100 (the default), and `no-unmatched-pair` stays off.

```toml
# ~/.config/jafix/config.toml
[builtin]
sentence-length = { max = 120 }
no-unmatched-pair = false
```

```toml
# .jafix.toml
[builtin]
sentence-length = { countBy = "codepoints" }
```

## The 10 default rules {#defaults}

| Rule ID                        | Action         | What it does                                                              |
| ------------------------------ | -------------- | ------------------------------------------------------------------------- |
| `fullwidth-paren`              | Fix            | Converts full-width parentheses to half-width                             |
| `space-between-ja-and-alnum`   | Fix            | Removes spaces between Japanese and half-width alphanumerics              |
| `space-between-ja-and-paren`   | Fix            | Removes spaces between Japanese and parentheses                           |
| `no-kangxi-radicals`           | Fix            | Replaces Kangxi radicals with the corresponding kanji                     |
| `no-nfd`                       | Fix and report | Composes separate dakuten and handakuten. Reports pairs it cannot compose |
| `no-invalid-control-character` | Fix            | Removes control characters                                                |
| `no-zero-width-spaces`         | Fix            | Removes zero-width spaces                                                 |
| `no-unmatched-pair`            | Report         | Unmatched brackets and quotation marks                                    |
| `ja-no-successive-word`        | Report         | The same word twice in a row                                              |
| `date-weekday-mismatch`        | Report         | A date with the wrong weekday                                             |

**Fix** rules are applied by `jafix fix` and the hooks. **Report** rules only flag findings.

## Turn individual rules on or off {#rules}

Use a [rule ID](./rules.md) as a key under `[builtin]`.

| Value           | Meaning                         |
| --------------- | ------------------------------- |
| `true`          | Enable with the default options |
| `false`         | Disable                         |
| `{ max = 120 }` | Enable with the given options   |

```toml
[builtin]
no-unmatched-pair = false # disable a default rule
common-misspellings = true # add a rule
sentence-length = { max = 120 } # add a rule with an option
```

Quote IDs that contain `@`, `/`, or `.`.

```toml
[builtin]
"@textlint-ja/no-synonyms" = true
"1.1.2.見出し" = true
```

## Enable rules in bulk with presets {#presets}

A preset is a key that groups several rules.
Its value is `true` or `false`.

| Preset                           | Rules it enables                                                                             |
| -------------------------------- | -------------------------------------------------------------------------------------------- |
| `preset-ja-technical-writing`    | 23 rules for [Japanese technical writing](./rules.md#technical-writing)                      |
| `@textlint-ja/preset-ai-writing` | 5 rules for [patterns common in AI writing](./rules.md#ai-writing)                           |
| `preset-ai-words-ja`             | `no-ai-words`                                                                                |
| `preset-ja-spacing`              | 9 rules for [spacing](./rules.md#spacing)                                                    |
| `preset-jtf-style`               | 33 rules based on the [Japan Translation Federation (JTF) style guide](./rules.md#jtf-style) |

Within one file, an individual rule setting takes precedence over a preset.

```toml
[builtin]
preset-ja-technical-writing = true
ja-no-weak-phrase = false
```

A preset in the project file also overrides individual rule settings in the global file.

## Exclude files {#exclude}

| Key              | Fixes   | Findings |
| ---------------- | ------- | -------- |
| `exclude`        | Skipped | Hidden   |
| `report-exclude` | Applied | Hidden   |

```toml
# .jafix.toml
exclude = ["generated/**"]
report-exclude = ["docs/bad-examples.md"]
```

Values are globs.
In the global file, use an absolute path or start the pattern with `~/` or `**/`.
On Windows, an absolute pattern can be written as `C:/work/docs/**`.
In the project file, relative patterns resolve from the directory of `.jafix.toml`.

## Invalid configuration {#errors}

An undefined key makes jafix print the reason and exit with code 2.
So does enabling two rules that require opposite styles, such as `fullwidth-paren` and `4.3.1.丸かっこ（）`.

```console
$ jafix check example.md
jafix: /path/to/.jafix.toml: [builtin]: `foo` は組み込みのルールのIDでもpresetの名前でもない
```

## More detail {#more}

- [Rules](./rules.md): rule IDs and behavior
- [Reference](https://github.com/mjun0812/jafix/blob/main/REFERENCE.md#設定ファイル) (Japanese): option defaults, `[filters]`, and glob resolution
