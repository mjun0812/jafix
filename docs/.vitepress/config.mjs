import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'jafix',
  srcExclude: ['adr/**'],
  locales: {
    root: {
      label: '日本語',
      lang: 'ja',
      description: '日本語のMarkdownを校正するCLI',
      themeConfig: {
        nav: [
          { text: 'ルール一覧', link: '/textlint-rules' },
          { text: 'ベンチマーク', link: '/benchmark-vs-textlint' },
        ],
        outline: { label: '目次', level: [2, 3] },
        langMenuLabel: '言語を切り替える',
        skipToContentLabel: '本文に移動',
        darkModeSwitchLabel: '外観',
        lightModeSwitchTitle: 'ライトモードに切り替える',
        darkModeSwitchTitle: 'ダークモードに切り替える',
        returnToTopLabel: 'ページの先頭に戻る',
      },
    },
    en: {
      label: 'English',
      lang: 'en',
      description: 'A command-line tool for proofreading Japanese Markdown',
      themeConfig: {
        nav: [
          { text: 'Rules', link: '/en/textlint-rules' },
          { text: 'Benchmark', link: '/en/benchmark-vs-textlint' },
        ],
        outline: { level: [2, 3] },
      },
    },
  },
})
