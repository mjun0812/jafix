---
layout: home
hero:
  name: jafix
  text: A CLI for proofreading Japanese text
  tagline: Fixes brackets and spacing, and reports what it cannot fix mechanically.
  actions:
    - theme: brand
      text: Quick start
      link: /en/quick-start
    - theme: alt
      text: Rules
      link: /en/rules
features:
  - title: Fixes and reports
    details: fix rewrites full-width parentheses and stray spaces. check also reports repeated words and unmatched brackets.
    link: /en/quick-start
    linkText: Try it
  - title: Works without configuration
    details: Ten rules are on by default. Choose which rules to use in TOML.
    link: /en/configuration
    linkText: Configure
  - title: Agent hooks
    details: Fixes the Markdown lines Claude Code or Codex just edited and returns findings to the agent.
    link: /en/hooks
    linkText: Register
  - title: Fast
    details: Runs as a single binary and finishes 14 to 142 times faster than textlint with the same rules.
    link: /en/benchmark-vs-textlint
    linkText: Measurements
---
