#!/usr/bin/env node
// docs/rule-examples/examples.json の例文をjafixに渡し、ルールごとの画像 (SVG) と「ルールの例」のページを、日本語版と英語版で作る。
// 画像の文字、色、文面は、実際の `jafix check` と `jafix fix` の結果から決める。手で描くと、ルールの挙動や文面が変わったときに、
// 画像だけが古くなる。
//
// 使い方 (先に `cargo build --locked` でバイナリを作る。バイナリは環境変数 JAFIX で変えられる):
//   node scripts/rule-examples.mjs                    画像とページを書き出す
//   node scripts/rule-examples.mjs --check            書き出さず、今あるファイルと同じ内容になるかを確かめる
//   node scripts/rule-examples.mjs --validate <file> [<dir>]  <file> の定義だけを確かめる。<dir> を渡すと、日本語版の画像をそこに書く
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const jafix = resolve(root, process.env.JAFIX ?? 'target/debug/jafix')

const WIDTH = 720
const MARGIN = 24
const TEXT_LEFT = 104
const TEXT_WIDTH = WIDTH - TEXT_LEFT - MARGIN
const FONT_SIZE = 22
const LINE_HEIGHT = 36
// 文字を1つずつ決まった幅のマスに置く。画像の中の文字は見る人の環境のフォントで描かれるので、文字の幅に任せると、
// 色を付けたマスと文字がずれる
const WIDE = FONT_SIZE
const NARROW = FONT_SIZE * 0.6
const MONO = 'ui-monospace,SFMono-Regular,Menlo,Consolas,monospace'
const SANS = '"Hiragino Sans","Noto Sans JP","Yu Gothic",Meiryo,"Apple Color Emoji","Segoe UI Emoji","Noto Color Emoji",sans-serif'
const KINDS = {
  fix: { fill: '#ddf4ff', color: '#0969da' },
  report: { fill: '#fff8c5', color: '#9a6700' },
}
const TONES = {
  removed: { fill: '#ffcecb', text: 'r', stroke: '#cf222e' },
  added: { fill: '#aceebb', text: 'g', stroke: '#116329' },
}
// 表示されない文字。画像ではコードポイントを書いた枠にする
const INVISIBLE = /^[\u0000-\u0008\u000b-\u001f\u007f-\u009f­​-‏‪-‮⁠-⁤﻿]/
// 行頭に置かない文字 (閉じの記号、句読点、小さいかな、長音符)。行の幅を少し超えても、前の行の末尾に置く
const NO_LINE_START = /^[、。，．！？）」』】〕］｝〉》)\]}!?,.…ーぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮ]/
// 行末に置かない開きの括弧。次の行の先頭へ送る
const NO_LINE_END = /^[「『（【〔［｛〈《(\[{]/
// 行が幅をこれだけ超えても、折り返さない。画像の右の余白に収まる
const HANG = 18
// ラベルと下の段に使うフォントの、半角の文字の幅の割合。広めに見積もって、右の件数に重ならないようにする
const SANS_NARROW = 0.54
const segmenter = new Intl.Segmenter('ja', { granularity: 'grapheme' })
// 画像とページは、日本語版と英語版で別に作る。例文は同じで、ラベルと下の段の文が違う
const LANGS = {
  ja: {
    images: 'docs/rule-examples',
    page: 'docs/rule-examples.md',
    rules: 'docs/rules.md',
    kinds: { fix: '修正', report: '指摘' },
    rows: { before: '修正前', after: '修正後', example: '例文' },
    count: (count) => `${count}か所`,
    title: (rule) => `${rule} の例`,
  },
  en: {
    images: 'docs/en/rule-examples',
    page: 'docs/en/rule-examples.md',
    rules: 'docs/en/rules.md',
    kinds: { fix: 'Fix', report: 'Report' },
    rows: { before: 'Before', after: 'After', example: 'Example' },
    count: (count) => (count === 1 ? '1 finding' : `${count} findings`),
    title: (rule) => `Example of ${rule}`,
  },
}
const INTRO = {
  ja: [
    '# ルールの例',
    '',
    'このページは、ルールが文章をどう変えるかを、ルールごとの画像で示します。画像は、例文を実際の `jafix` に渡した結果から作っています。ルールの種類と初期状態は[ルール一覧](./rules.md)で確認できます。',
    '',
    '## 画像の見方 {#legend}',
    '',
    '画像は、上から順に、ルールIDと種類、例文、指摘の文面を示します。例文には、そのルールだけを有効にした設定を使っています。オプションを変えた例には、その設定を添えています。',
    '',
    '- 種類が「修正」の画像は、修正前と修正後の2行です。赤い背景は修正で消える文字、緑の背景は修正で入る文字です',
    '- 種類が「指摘」の画像は、例文だけです。赤い背景は、指摘の対象の語句です',
    '- 底の開いた箱の形は、空白です。空白は見えないので、形に置き換えています',
    '- `U+200B` のような枠は、表示されない文字です。枠の中は、その文字のコードポイントです',
    '- `↵` は改行です',
    '- 下の段は、`jafix check` が出す文面と、指摘の数です',
  ],
  en: [
    '# Rule examples',
    '',
    "This page shows how each rule changes text, with one image per rule. Every image is generated from the output of the real `jafix` binary for the example text. See [Rules](./rules.md) for each rule's kind and default state.",
    '',
    '## How to read the images {#legend}',
    '',
    'From top to bottom, an image shows the rule ID and kind, the example text, and what the rule does. Each example runs with only that rule enabled. When an example changes an option, the configuration follows the image.',
    '',
    '- A `Fix` image has two rows, `Before` and `After`. A red background marks characters the fix removes, and a green background marks characters it inserts',
    '- A `Report` image has the example text only. A red background marks the reported words',
    '- An open box stands for a space, which would otherwise be invisible',
    '- A frame such as `U+200B` stands for a character that is not displayed. It shows the code point of the character',
    '- `↵` is a line break',
    '- The bottom line describes the rule and gives the number of findings. `jafix check` itself prints its messages in Japanese',
  ],
}

const [mode, target, preview] = modeOf(process.argv.slice(2))
const source = mode === 'validate' ? resolve(target) : join(root, LANGS.ja.images, 'examples.json')
const { sections, skipped = [] } = JSON.parse(readFileSync(source, 'utf8'))
const examples = sections.flatMap((section) => section.rules)
// 利用者の設定を読まないよう、設定の置き場と作業ディレクトリを空の一時ディレクトリへ向ける
const home = mkdtempSync(join(tmpdir(), 'jafix-rule-examples-'))
const files = new Map()
try {
  const rules = JSON.parse(run(['rules', '--output-format', 'json'])).rules
  for (const example of examples) Object.assign(example, check(example, rules))
  const slugs = examples.map(({ rule }) => slug(rule))
  const repeated = examples.find(({ rule }, index) => slugs.indexOf(slug(rule)) !== index)
  if (repeated) throw new Error(`${repeated.rule}: 画像のファイル名がほかのルールと重なる`)
  if (mode === 'validate') {
    for (const example of examples) {
      console.log(`${example.rule}: ${LANGS.ja.kinds[example.kind]} ${example.findings.length}か所`)
      if (preview) writeFileSync(join(resolve(preview), `${slug(example.rule)}.svg`), render(example, 'ja', example.findings[0].message))
    }
  } else {
    checkCoverage(rules)
    for (const lang of Object.keys(LANGS)) {
      const descriptions = descriptionsIn(lang)
      for (const example of examples) {
        // 英語版の下の段には、ルール一覧の説明を書く。jafixの文面は日本語しか無い
        const footer = lang === 'ja' ? example.findings[0].message : descriptions.get(example.rule)
        files.set(`${LANGS[lang].images}/${slug(example.rule)}.svg`, render(example, lang, footer))
      }
      files.set(LANGS[lang].page, page(lang))
    }
  }
} finally {
  rmSync(home, { recursive: true, force: true })
}
if (mode === 'write') write()
if (mode === 'check') compare()

function modeOf(args) {
  if (args.length === 0) return ['write']
  if (args.length === 1 && args[0] === '--check') return ['check']
  if ((args.length === 2 || args.length === 3) && args[0] === '--validate') return ['validate', args[1], args[2]]
  console.error('usage: rule-examples.mjs [--check | --validate <file> [<dir>]]')
  process.exit(2)
}

function run(args, input = '') {
  const env = { ...process.env, HOME: home, XDG_CONFIG_HOME: home }
  return execFileSync(jafix, args, { input, encoding: 'utf8', cwd: home, env, maxBuffer: 1 << 26 })
}

// 例文を、そのルールだけを有効にした設定で確かめる。ほかのルールの指摘が混ざると、画像がそのルールの効果を表さなくなる
function check({ rule, text, value = 'true', mark }, rules) {
  if (!rules.some(({ id }) => id === rule)) throw new Error(`${rule}: 組み込みのルールに無い`)
  const others = rules.filter(({ id, enabled }) => enabled && id !== rule).map(({ id }) => `${key(id)} = false\n`)
  mkdirSync(join(home, 'jafix'), { recursive: true })
  writeFileSync(join(home, 'jafix/config.toml'), `[builtin]\n${others.join('')}${key(rule)} = ${value}\n`)
  let checked
  try {
    checked = run(['check', '--output-format', 'json', '-'], `${text}\n`)
  } catch (error) {
    // 指摘があると終了コード1になる。それ以外は、設定か入力の誤りである
    if (error.status !== 1) throw new Error(`${rule}: ${error.stderr || error.message}`)
    checked = error.stdout
  }
  const { findings } = JSON.parse(checked)
  if (findings.length === 0) throw new Error(`${rule}: 例文に指摘が出ない: ${text}`)
  const fixed = run(['fix', '-'], `${text}\n`).replace(/\n$/, '')
  if (fixed !== text) return { kind: 'fix', findings, fixed }
  return { kind: 'report', findings, marked: markedBy(rule, text, findings, mark) }
}

// 指摘ルールの画像で色を付ける位置。`jafix check` は指摘の始まりの位置だけを出すので、対象の語句は定義の `mark` に書く。
// 語句が、報告された行と列から始まることを確かめる
function markedBy(rule, text, findings, mark) {
  const marks = [mark ?? []].flat()
  if (marks.length !== findings.length) {
    const found = findings.map(({ line, column, message }) => `${line}:${column} ${message}`).join(' / ')
    throw new Error(`${rule}: mark には、指摘と同じ数 (${findings.length}個) の語句を書く。指摘: ${found}`)
  }
  const lines = text.split('\n')
  const marked = new Set()
  findings.forEach(({ line, column }, index) => {
    // 列は文字 (コードポイント) で数える
    const before = [...lines.slice(0, line - 1).join('\n')].length + (line > 1 ? 1 : 0) + column - 1
    const rest = [...text].slice(before).join('')
    if (!rest.startsWith(marks[index])) {
      throw new Error(`${rule}: ${line}:${column} は「${rest.slice(0, 12)}」から始まる。mark の「${marks[index]}」と合わない`)
    }
    let offset = 0
    graphemes(text).forEach((grapheme, position) => {
      const end = offset + [...grapheme].length
      if (end > before && offset < before + [...marks[index]].length) marked.add(position)
      offset = end
    })
  })
  return marked
}

// 組み込みのルールは、どれも例を持つか、例を作らない理由を `skipped` に持つ
function checkCoverage(rules) {
  const listed = new Set([...examples, ...skipped].map(({ rule }) => rule))
  const missing = rules.filter(({ id }) => !listed.has(id)).map(({ id }) => id)
  if (missing.length > 0) throw new Error(`例の無いルールがある。examples.json の sections か skipped に足す: ${missing.join(', ')}`)
  const unknown = skipped.filter(({ rule }) => !rules.some(({ id }) => id === rule)).map(({ rule }) => rule)
  if (unknown.length > 0) throw new Error(`skipped に、組み込みのルールに無いIDがある: ${unknown.join(', ')}`)
}

// ルール一覧の表から、ルールごとの説明を読む。あわせて、表のルールIDが「ルールの例」の該当の箇所へリンクしていることを確かめる
function descriptionsIn(lang) {
  const anchors = new Map([...examples.map(({ rule }) => [rule, slug(rule)]), ...skipped.map(({ rule }) => [rule, 'skipped'])])
  const descriptions = new Map()
  const unlinked = []
  for (const line of readFileSync(join(root, LANGS[lang].rules), 'utf8').split('\n')) {
    const cells = line.split('|').slice(1, -1).map((cell) => cell.trim())
    const rule = cells[0]?.match(/^\[?`([^`]+)`/)?.[1]
    if (!anchors.has(rule)) continue
    const link = `[\`${rule}\`](./rule-examples.md#${anchors.get(rule)})`
    if (cells[0] !== link) unlinked.push(link)
    // 画像には文字だけを書くので、Markdownの記号を除く
    descriptions.set(rule, cells.at(-1).replaceAll('`', '').replace(/\[([^\]]+)\]\([^)]+\)/g, '$1'))
  }
  if (unlinked.length > 0) throw new Error(`${LANGS[lang].rules} のルールIDを、次のリンクにする:\n${unlinked.join('\n')}`)
  const missing = [...anchors.keys()].filter((rule) => !descriptions.has(rule))
  if (missing.length > 0) throw new Error(`${LANGS[lang].rules} の表に無いルールがある: ${missing.join(', ')}`)
  return descriptions
}

function render({ rule, text, kind, findings, fixed, marked }, lang, footer) {
  const { kinds, rows: labels, count, title } = LANGS[lang]
  let rows
  if (kind === 'fix') {
    const [removed, added] = diff(graphemes(text), graphemes(fixed))
    rows = [
      { label: labels.before, text, lines: layout(text, removed), tone: TONES.removed },
      { label: labels.after, text: fixed, lines: layout(fixed, added), tone: TONES.added },
    ]
  } else {
    rows = [{ label: labels.example, text, lines: layout(text, marked), tone: TONES.removed }]
  }
  const top = 46
  const body = []
  let y = top + 14
  for (const row of rows) {
    body.push(`<text class="l" x="${MARGIN}" y="${y + 23}">${row.label}</text>`)
    for (const line of row.lines) {
      body.push(...renderLine(line, y + 24, row.tone))
      y += LINE_HEIGHT
    }
    y += 12
  }
  const footerTop = y + 4
  const counted = count(findings.length)
  const message = wrap(footer, 13, WIDTH - MARGIN * 2 - widthOf(counted, 13, SANS_NARROW) - 24)
  const height = footerTop + 24 + message.length * 20
  const badge = KINDS[kind]
  const badgeLeft = MARGIN + widthOf(rule, 15) + 12
  const badgeWidth = widthOf(kinds[kind], 12, SANS_NARROW) + 20
  return `${[
    `<svg xmlns="http://www.w3.org/2000/svg" width="${WIDTH}" height="${height}" viewBox="0 0 ${WIDTH} ${height}" role="img" aria-labelledby="title desc">`,
    `<title id="title">${escape(title(rule))}</title>`,
    `<desc id="desc">${escape(rows.map((row) => `${row.label}: ${plain(row.text)}`).join(' / '))}</desc>`,
    `<style>text{fill:#1f2328}.w{font:${FONT_SIZE}px ${SANS};text-anchor:middle}.n{font:${FONT_SIZE}px ${MONO};text-anchor:middle}.c{font:11px ${MONO};text-anchor:middle}.l{font:13px ${SANS};fill:#59636e}.r{fill:#cf222e}.g{fill:#116329}.q{fill:#8c959f}</style>`,
    `<rect x="0.5" y="0.5" width="${WIDTH - 1}" height="${height - 1}" rx="8" fill="#ffffff" stroke="#d1d9e0"/>`,
    `<path d="M1 ${top}V9a8 8 0 0 1 8-8h${WIDTH - 18}a8 8 0 0 1 8 8v${top - 9}z" fill="#f6f8fa"/>`,
    `<path d="M1 ${top}.5h${WIDTH - 2}M1 ${footerTop}.5h${WIDTH - 2}" stroke="#d1d9e0"/>`,
    `<text x="${MARGIN}" y="29" style='font:600 15px ${MONO}'>${escape(rule)}</text>`,
    `<rect x="${round(badgeLeft)}" y="13" width="${round(badgeWidth)}" height="22" rx="11" fill="${badge.fill}"/>`,
    `<text x="${round(badgeLeft + badgeWidth / 2)}" y="28.5" style='font:600 12px ${SANS};text-anchor:middle;fill:${badge.color}'>${kinds[kind]}</text>`,
    ...body,
    ...message.map((line, index) => `<text class="l" x="${MARGIN}" y="${footerTop + 26 + index * 20}">${escape(line)}</text>`),
    `<text class="l" x="${WIDTH - MARGIN}" y="${footerTop + 26}" style="text-anchor:end">${counted}</text>`,
    '</svg>',
  ].join('\n')}\n`
}

// 文字を行に並べる。元の改行で行を分け、幅に収まらない行は折り返す
function layout(text, marked) {
  const sourceLines = [[]]
  graphemes(text).forEach((grapheme, position) => {
    sourceLines.at(-1).push({ ...cellOf(grapheme), grapheme, hit: marked.has(position) })
    if (grapheme === '\n') sourceLines.push([])
  })
  return sourceLines.flatMap((cells) => {
    // 幅いっぱいまで詰めると、最後の行に1〜2文字だけが残ることがある。最後の行が短くなりすぎるときは、前の行を少し短くする
    const total = cells.reduce((sum, { width }) => sum + width, 0)
    const count = Math.max(1, Math.ceil((total - HANG) / TEXT_WIDTH))
    const shortage = Math.max(0, TEXT_WIDTH * 0.3 - (total - TEXT_WIDTH * (count - 1)))
    const limit = count === 1 ? Infinity : TEXT_WIDTH - shortage / (count - 1)
    const lines = [[]]
    let x = 0
    for (const cell of cells) {
      if (x + cell.width > limit && x > 0 && !NO_LINE_START.test(cell.grapheme)) {
        const carried = NO_LINE_END.test(lines.at(-1).at(-1).grapheme) ? [lines.at(-1).pop()] : []
        lines.push(carried.map((moved) => ({ ...moved, x: TEXT_LEFT })))
        x = carried.reduce((sum, { width }) => sum + width, 0)
      }
      lines.at(-1).push({ ...cell, x: TEXT_LEFT + x })
      x += cell.width
    }
    return lines
  })
}

function cellOf(grapheme) {
  if (grapheme === '\n') return { type: 'newline', width: WIDE }
  if (grapheme === ' ') return { type: 'space', width: NARROW }
  if (grapheme === '　') return { type: 'space', width: WIDE }
  const code = grapheme.codePointAt(0)
  if (INVISIBLE.test(grapheme)) {
    const label = `U+${code.toString(16).toUpperCase().padStart(4, '0')}`
    return { type: 'code', label, width: label.length * 6.8 + 10 }
  }
  if (grapheme === '\t') return { type: 'code', label: 'TAB', width: 30.4 }
  // 半角の文字は、ラテン文字と半角カナである。半角カナの濁点と半濁点は、前の文字と1つにまとまるが、幅は1文字分ある
  const narrow = code < 0x300 || (code >= 0xff61 && code <= 0xff9f)
  const voiced = grapheme.match(/[ﾞﾟ]/g)?.length ?? 0
  return { type: 'char', width: (narrow ? NARROW : WIDE) + (code < 0xff9e ? voiced * NARROW : 0), font: code < 0x80 ? 'n' : 'w' }
}

function renderLine(cells, baseline, tone) {
  return cells.flatMap(({ type, grapheme, label, width, font, hit, x }) => {
    const parts = hit
      ? [`<rect x="${round(x)}" y="${baseline - 21}" width="${round(width)}" height="29" rx="3" fill="${tone.fill}"/>`]
      : []
    const center = round(x + width / 2)
    if (type === 'space') {
      // 空白は見えないので、底の開いた箱の形で示す
      const stroke = hit ? tone.stroke : '#afb8c1'
      parts.push(`<path d="M${round(x + 2.5)} ${baseline - 3}v5h${round(width - 5)}v-5" fill="none" stroke="${stroke}" stroke-width="1.5"/>`)
    } else if (type === 'newline') {
      parts.push(`<text class="w ${hit ? tone.text : 'q'}" x="${center}" y="${baseline}" style="font-size:16px">↵</text>`)
    } else if (type === 'code') {
      const stroke = hit ? tone.stroke : '#8c959f'
      parts.push(
        `<rect x="${round(x + 2)}" y="${baseline - 15}" width="${round(width - 4)}" height="18" rx="3" fill="none" stroke="${stroke}"/>`,
        `<text class="c ${hit ? tone.text : 'q'}" x="${center}" y="${baseline - 2}">${label}</text>`,
      )
    } else {
      parts.push(`<text class="${font}${hit ? ` ${tone.text}` : ''}" x="${center}" y="${baseline}">${escape(grapheme)}</text>`)
    }
    return parts
  })
}

// 修正前と修正後を文字ごとに比べ、消えた文字の位置と入った文字の位置を返す (最長共通部分列)
function diff(before, after) {
  const lengths = Array.from({ length: before.length + 1 }, () => new Array(after.length + 1).fill(0))
  for (let i = before.length - 1; i >= 0; i--) {
    for (let j = after.length - 1; j >= 0; j--) {
      lengths[i][j] = before[i] === after[j] ? lengths[i + 1][j + 1] + 1 : Math.max(lengths[i + 1][j], lengths[i][j + 1])
    }
  }
  const removed = new Set()
  const added = new Set()
  let i = 0
  let j = 0
  while (i < before.length || j < after.length) {
    if (i < before.length && j < after.length && before[i] === after[j]) {
      i++
      j++
    } else if (j === after.length || (i < before.length && lengths[i + 1][j] >= lengths[i][j + 1])) {
      removed.add(i++)
    } else {
      added.add(j++)
    }
  }
  return [removed, added]
}

function page(lang) {
  const ja = lang === 'ja'
  const parts = [
    '---',
    'outline: 2',
    '---',
    '',
    '<!-- このファイルは scripts/rule-examples.mjs が docs/rule-examples/examples.json から生成する。直接編集しない -->',
    '',
    ...INTRO[lang],
  ]
  for (const section of sections) {
    parts.push('', `## ${ja ? section.title : section.title_en} {#${section.id}}`)
    for (const { rule, value, note, note_en: noteEn } of section.rules) {
      // ルールIDには `<>` などの記号があるので、コードとして書く
      parts.push('', `### \`${rule}\` {#${slug(rule)}}`, '')
      const text = ja ? note : noteEn
      if (text) parts.push(text, '')
      parts.push(`![${LANGS[lang].title(rule)}](./rule-examples/${slug(rule)}.svg)`)
      if (value) parts.push('', ja ? 'この例の設定は、次のとおりです。' : 'This example uses the following configuration.', '', '```toml', '[builtin]', `${key(rule)} = ${value}`, '```')
    }
  }
  if (skipped.length > 0) {
    parts.push('', ja ? '## 例の無いルール {#skipped}' : '## Rules without an example {#skipped}', '')
    parts.push(ja ? '次のルールは、1つの例文では示せないので、画像がありません。' : 'The following rules have no image because a single example text cannot show them.', '')
    for (const { rule, reason, reason_en: reasonEn } of skipped) parts.push(`- \`${rule}\`: ${ja ? reason : reasonEn}`)
  }
  return `${parts.join('\n')}\n`
}

function write() {
  for (const { images } of Object.values(LANGS)) mkdirSync(join(root, images), { recursive: true })
  for (const [path, content] of files) writeFileSync(join(root, path), content)
  for (const path of stale()) rmSync(join(root, path))
  console.log(`${files.size - 2}個の画像と、2つのページを書き出した`)
}

function compare() {
  const changed = [...files].filter(([path, content]) => !existsSync(join(root, path)) || readFileSync(join(root, path), 'utf8') !== content)
  const problems = [...changed.map(([path]) => path), ...stale()]
  if (problems.length === 0) return
  console.error(`次のファイルが、今のjafixの結果と合わない。\`node scripts/rule-examples.mjs\` で作り直す:\n${problems.join('\n')}`)
  process.exit(1)
}

// 定義から消えたルールの画像
function stale() {
  return Object.values(LANGS)
    .filter(({ images }) => existsSync(join(root, images)))
    .flatMap(({ images }) => readdirSync(join(root, images)).map((name) => `${images}/${name}`))
    .filter((path) => path.endsWith('.svg') && !files.has(path))
}

// 画像の説明文に書く例文。表示されない文字はコードポイントにする (制御文字は、XMLに書けない)
function plain(text) {
  return graphemes(text)
    .map((grapheme) => (INVISIBLE.test(grapheme) ? cellOf(grapheme).label : grapheme))
    .join('')
}

function graphemes(text) {
  return [...segmenter.segment(text)].map(({ segment }) => segment)
}

// 下の段の文を折り返す。英数字の語の途中では折り返さない
function wrap(text, size, max) {
  const lines = ['']
  let width = 0
  for (const token of text.match(/[!-˿]+|[\s\S]/gu)) {
    const tokenWidth = widthOf(token, size, SANS_NARROW)
    if (width + tokenWidth > max && width > 0 && !NO_LINE_START.test(token)) {
      lines.push('')
      width = 0
      if (token === ' ') continue
    }
    lines[lines.length - 1] += token
    width += tokenWidth
  }
  return lines
}

// 文字列の幅の見積もり。`narrow` は、半角の文字の幅の、文字の大きさに対する割合 (等幅のフォントは0.6)
function widthOf(text, size, narrow = 0.6) {
  return graphemes(text).reduce((sum, grapheme) => sum + (grapheme.codePointAt(0) < 0x300 ? size * narrow : size), 0)
}

// 画像のファイル名と見出しのアンカー。ルールIDの `@` `/` や日本語を除く
function slug(rule) {
  return rule
    .replace(/[^A-Za-z0-9.-]+/g, '-')
    .replace(/^[-.]+|[-.]+$/g, '')
    .toLowerCase()
}

function key(rule) {
  return /^[A-Za-z0-9_-]+$/.test(rule) ? rule : JSON.stringify(rule)
}

function round(value) {
  return Math.round(value * 10) / 10
}

function escape(text) {
  return text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;')
}
