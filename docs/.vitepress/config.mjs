import { defineConfig } from 'vitepress'

const repo = 'https://github.com/mjun0812/jafix'

export default defineConfig({
  title: 'jafix',
  srcExclude: ['adr/**'],
  themeConfig: {
    socialLinks: [{ icon: 'github', link: repo }],
  },
  locales: {
    root: {
      label: '日本語',
      lang: 'ja',
      description: '日本語に特化した高速なtextlint',
      themeConfig: {
        nav: [
          { text: 'ガイド', link: '/quick-start', activeMatch: '^/(quick-start|configuration|hooks)' },
          { text: 'ルール一覧', link: '/rules' },
          { text: 'ベンチマーク', link: '/benchmark-vs-textlint' },
        ],
        sidebar: [
          {
            text: 'ガイド',
            items: [
              { text: 'クイックスタート', link: '/quick-start' },
              { text: '設定', link: '/configuration' },
              { text: 'hookへの登録', link: '/hooks' },
            ],
          },
          {
            text: '資料',
            items: [
              { text: 'ルール一覧', link: '/rules' },
              { text: 'ベンチマーク', link: '/benchmark-vs-textlint' },
              { text: 'リファレンス(GitHub)', link: `${repo}/blob/main/REFERENCE.md` },
            ],
          },
        ],
        outline: { label: '目次', level: [2, 3] },
        docFooter: { prev: '前のページ', next: '次のページ' },
        sidebarMenuLabel: 'メニュー',
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
      description: 'A fast textlint for Japanese prose',
      themeConfig: {
        nav: [
          {
            text: 'Guide',
            link: '/en/quick-start',
            activeMatch: '^/en/(quick-start|configuration|hooks)',
          },
          { text: 'Rules', link: '/en/rules' },
          { text: 'Benchmark', link: '/en/benchmark-vs-textlint' },
        ],
        sidebar: [
          {
            text: 'Guide',
            items: [
              { text: 'Quick start', link: '/en/quick-start' },
              { text: 'Configuration', link: '/en/configuration' },
              { text: 'Hooks', link: '/en/hooks' },
            ],
          },
          {
            text: 'Reference',
            items: [
              { text: 'Rules', link: '/en/rules' },
              { text: 'Benchmark', link: '/en/benchmark-vs-textlint' },
              { text: 'Full reference (Japanese, GitHub)', link: `${repo}/blob/main/REFERENCE.md` },
            ],
          },
        ],
        outline: { level: [2, 3] },
      },
    },
  },
})
