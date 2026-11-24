---
layout: home
hero:
  name: jafix
  text: A fast textlint for Japanese prose
  tagline: Lint and autofix prose with a single Rust binary.
  actions:
    - theme: brand
      text: Quick start
      link: /en/quick-start
    - theme: alt
      text: Rules
      link: /en/rules
benchmark:
  title: Benchmark against textlint
  sizeLabel: File size
  ratioLabel: Ratio
  ratioUnit: ×
  link: /en/benchmark-vs-textlint
  linkText: Details
features:
  - title: Check and autofix
    details: check reports findings without changing your files. fix saves the changes it can apply automatically.
    link: /en/quick-start
    linkText: Try it
  - title: Works without configuration
    details: Ten rules are on by default. Choose which rules to use in TOML.
    link: /en/configuration
    linkText: Configure
  - title: Agent hooks
    details: Automatically fixes lines Claude Code or Codex just added or changed in Markdown, then reports remaining findings to the agent.
    link: /en/hooks
    linkText: Register
---
