# Performance comparison with textlint {#benchmark}

We configured jafix and textlint to perform the same checks and fixes, then timed three workflows: `check`, `fix`, and a hook run.
Across the tested workflows and file sizes, jafix finished 14 to 142 times faster and used at most one third as much memory.
These measurements were taken on November 15, 2026, using `scripts/bench-vs-textlint.sh`.

## Method {#method}

Both configurations covered the same five checks and fixes.
jafix used three built-in fix rules and two entries under `@textlint-rule/pattern`.
The table shows the corresponding textlint rules.

| Operation                                                             | jafix                        | textlint                                                |
| --------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------- |
| Convert full-width parentheses to half-width                          | `fullwidth-paren`            | `prh`                                                   |
| Remove spaces between Japanese characters and Latin letters or digits | `space-between-ja-and-alnum` | `ja-space-between-half-and-full-width` (`space: never`) |
| Remove spaces between Japanese characters and parentheses             | `space-between-ja-and-paren` | A replacement in `@textlint-rule/pattern`               |
| Flag words from a list of 30                                          | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |
| Flag runs of six or more kanji                                        | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |

The `check` workflow reports findings and leaves the file intact.
The `fix` workflow applies fixes to the entire file and writes them back.
For the hook workflow, we passed `jafix hook claude` input representing a Claude Code Edit that changed 20 lines starting at line 50.
textlint processes the whole file, so its equivalent workflow ran `textlint --fix` and then `textlint` to report findings.

We used four input files.
We created three by repeating a Markdown block of about 350 bytes 10, 100, and 1,000 times.
The block contains headings and paragraphs as well as lists, emphasis, and code blocks.
It exercises all five rules.
The fourth input was jafix's `README.md`.
We used hyperfine to run three warmups and average 20 measurements, or 10 for the 343 KB file.

## Comparing the work performed {#work-performed}

Counts were similar across the five operations.
In the 3 KB file, the tools found the same number of full-width parentheses, spaces next to parentheses, and matches for the two report rules.
jafix found 30 more spaces between Japanese and Latin text.

| Operation                                                       | jafix | textlint |
| --------------------------------------------------------------- | ----: | -------: |
| Full-width parentheses                                          |    20 |       20 |
| Spaces between Japanese characters and Latin letters or digits  |   120 |       90 |
| Spaces next to parentheses and matches for the two report rules |    70 |       70 |

The extra matches include spaces after a long vowel mark in `サーバー API` and around a number in `系 38`.
jafix removed those spaces, which gave it more work in the comparison.

## Results {#results}

The measurements used an Apple M5 running macOS on arm64, a release build of jafix 0.1.0 (00b728a), textlint v15.8.0 on Node.js v26.13.0, and hyperfine 2.0.0.
Times are mean wall-clock times, memory is mean peak resident memory, and speedup is textlint's time divided by jafix's time.

| Workflow | File                   |          jafix |         textlint | Speedup |
| -------- | ---------------------- | -------------: | ---------------: | ------: |
| check    | blocks-10.md (3KB)     |    2.6ms / 7MB |  151.7ms / 122MB |     59× |
| check    | blocks-100.md (34KB)   |   9.2ms / 14MB |  249.8ms / 189MB |     27× |
| check    | README.md (69KB)       |  18.7ms / 12MB |  267.8ms / 178MB |     14× |
| check    | blocks-1000.md (343KB) | 120.1ms / 79MB |   2755ms / 328MB |     23× |
| fix      | blocks-10.md (3KB)     |    2.3ms / 6MB |  167.8ms / 136MB |     73× |
| fix      | blocks-100.md (34KB)   |   6.1ms / 13MB |  330.6ms / 203MB |     54× |
| fix      | README.md (69KB)       |   5.7ms / 12MB |  262.5ms / 185MB |     46× |
| fix      | blocks-1000.md (343KB) |  89.5ms / 78MB | 4464.4ms / 406MB |     50× |
| hook     | blocks-10.md (3KB)     |    2.9ms / 7MB |  411.1ms / 135MB |    142× |
| hook     | blocks-100.md (34KB)   |  11.2ms / 14MB |   1031ms / 202MB |     92× |
| hook     | README.md (69KB)       |  10.5ms / 12MB |  490.7ms / 188MB |     47× |
| hook     | blocks-1000.md (343KB) | 192.6ms / 92MB | 6319.1ms / 408MB |     33× |

## What the results show {#interpretation}

The gap was largest for the smallest file, at 3 KB.
Starting Node.js and loading its rules took textlint about 150 ms per invocation.
The same part took jafix about 2 ms.
For files ranging from a few kilobytes to a few dozen kilobytes, the size an agent might edit in one operation, startup accounts for much of the runtime.

For small files, the hook workflow had the largest gap.
textlint paid the startup cost twice and parsed the whole file both times.
jafix applied fixes and collected findings in one process, and limited its fixes to the edited lines.

The hook speedup narrowed from 142× at 3 KB to 33× at 343 KB.
The results between 34 KB and 343 KB showed no consistent size trend for `check` and `fix`, with speedups ranging from 14× to 54×.
At 343 KB, jafix was still 23 to 50 times faster.
The hook parses the entire document even for a small edit, so its runtime grows with file size.

Memory use also differed.
jafix used around 6 to 7 MB for files of a few kilobytes and up to 92 MB at 343 KB.
textlint used at least 120 MB in every run.

These measurements cover runs after warmup.
textlint takes longer on a cold run because the files have not yet been cached.

## Reproducing the benchmark {#reproduce}

Run `scripts/bench-vs-textlint.sh` to print the results table.
Pass a directory to save hyperfine's JSON results there.

```sh
scripts/bench-vs-textlint.sh /tmp/bench-results
```

Set `TEXTLINT_PREFIX` to the directory containing `<prefix>/bin/textlint` and `<prefix>/node_modules`.
The script uses `mise where npm:textlint` by default.
Install `textlint-rule-prh`, `textlint-rule-ja-space-between-half-and-full-width`, and `@textlint-rule/textlint-rule-pattern` into that prefix.
The script also requires `hyperfine` and `jq`.
