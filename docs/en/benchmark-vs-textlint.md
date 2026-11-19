# Performance comparison with textlint {#benchmark}

Against textlint with the same rules, jafix finished 14 to 142 times faster and used at most one third of the memory.
We timed three workflows (`check`, `fix`, and a hook run) on four files from 3 KB to 343 KB.
The measurements were taken on November 15, 2026, with `scripts/bench-vs-textlint.sh`.

## Method {#method}

Both tools were configured with five rules that do the same work.

| Operation                                                             | jafix                        | textlint                                                |
| --------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------- |
| Convert full-width parentheses to half-width                          | `fullwidth-paren`            | `prh`                                                   |
| Remove spaces between Japanese characters and Latin letters or digits | `space-between-ja-and-alnum` | `ja-space-between-half-and-full-width` (`space: never`) |
| Remove spaces between Japanese characters and parentheses             | `space-between-ja-and-paren` | A replacement in `@textlint-rule/pattern`               |
| Flag words from a list of 30                                          | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |
| Flag runs of six or more kanji                                        | `@textlint-rule/pattern`     | `@textlint-rule/pattern`                                |

We timed these commands for each workflow.

| Workflow | jafix               | textlint                          |
| -------- | ------------------- | --------------------------------- |
| check    | `jafix check`       | `textlint`                        |
| fix      | `jafix fix`         | `textlint --fix`                  |
| hook     | `jafix hook claude` | `textlint --fix`, then `textlint` |

The hook input represents a Claude Code Edit that rewrote 20 lines starting at line 50.
textlint cannot read hook input or limit fixes to certain lines.
We therefore counted two runs over the whole file as one hook run.

We used four input files.

| File             | Size   | Content                                       |
| ---------------- | ------ | --------------------------------------------- |
| `blocks-10.md`   | 3 KB   | A block of about 350 bytes, repeated 10 times |
| `blocks-100.md`  | 34 KB  | The same block, repeated 100 times            |
| `blocks-1000.md` | 343 KB | The same block, repeated 1,000 times          |
| `README.md`      | 69 KB  | The jafix README                              |

The block contains headings and paragraphs as well as lists, emphasis, and code blocks.
It matches every rule.
With hyperfine, we ran three warmups and then averaged 20 runs (10 for the 343 KB file).

## Matching counts {#work-performed}

The number of matches per rule was nearly the same.
In the 3 KB file, the only difference was in spaces between Japanese and alphanumerics, where jafix matched 30 more.

| Operation                                                       | jafix | textlint |
| --------------------------------------------------------------- | ----: | -------: |
| Full-width parentheses                                          |    20 |       20 |
| Spaces between Japanese characters and Latin letters or digits  |   120 |       90 |
| Spaces next to parentheses and matches for the two report rules |    70 |       70 |

textlint's `ja-space-between-half-and-full-width` leaves the space after a long vowel mark (`サーバー API`) and after a kanji (`系 38`).
jafix handles more spots, so this difference works against jafix in the comparison.

## Results {#results}

| Item      | Value                            |
| --------- | -------------------------------- |
| Machine   | Apple M5 (macOS, arm64)          |
| jafix     | Release build of 0.1.0 (00b728a) |
| textlint  | v15.8.0 (Node.js v26.13.0)       |
| hyperfine | 2.0.0                            |

Times are mean wall-clock times, and memory is mean peak resident memory.
Speedup is textlint's time divided by jafix's time.

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

## Speedup trends {#interpretation}

The gap is largest for the 3 KB file.
On each run, textlint spends about 150 ms starting Node.js and loading its rules.
jafix spends about 2 ms on the same part.
For the files of a few to a few dozen kilobytes that an agent writes in one edit, this fixed time accounts for most of the runtime.

For files of a few kilobytes, the hook workflow has the largest gap.
textlint fixes and reports in separate runs, so it pays the startup time twice and parses the whole file twice.
jafix fixes and then collects findings in one process, and it fixes only the lines changed by the edit.

The hook speedup narrows as files grow, from 142× at 3 KB to 33× at 343 KB.
Parsing the text comes to outweigh the fixed time.
jafix parses the whole file even when few lines changed, so its hook time grows in proportion to file size.
For `check` and `fix`, the speedup between 34 KB and 343 KB ranges from 14× to 54× and shows no trend with size.

jafix used 6 to 7 MB for files of a few kilobytes and stayed within 92 MB at 343 KB.
textlint used at least 120 MB for every file.

All values come from runs after warmup.
textlint's first run cannot use the disk cache, so it takes longer.

## Reproducing the benchmark {#reproduce}

`scripts/bench-vs-textlint.sh` prints the same tables to standard output.
Pass a directory to keep hyperfine's JSON results there.

```sh
scripts/bench-vs-textlint.sh /tmp/bench-results
```

| Requirement | How to provide it                                                                                                                             |
| ----------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| textlint    | Set `TEXTLINT_PREFIX` to the prefix. If unset, the script uses the location from `mise where npm:textlint`                                    |
| Rules       | Install `textlint-rule-prh`, `textlint-rule-ja-space-between-half-and-full-width`, and `@textlint-rule/textlint-rule-pattern` into the prefix |
| Commands    | `hyperfine` and `jq`                                                                                                                          |

The prefix is a directory that contains `<prefix>/bin/textlint` and `<prefix>/node_modules`.
