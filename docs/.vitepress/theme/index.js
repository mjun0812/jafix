import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import HomeBenchmark from './HomeBenchmark.vue'
import HomeBenchmarkChart from './HomeBenchmarkChart.vue'

export default {
  extends: DefaultTheme,
  Layout: () => h(DefaultTheme.Layout, null, {
    'home-hero-image': () => h(HomeBenchmarkChart),
    'home-hero-after': () => h(HomeBenchmark),
  }),
}
