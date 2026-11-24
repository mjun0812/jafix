<script setup>
import { useData } from 'vitepress'
import { checkResults } from './benchmark-results.js'

const { frontmatter } = useData()
const sample = checkResults[0]
const tools = ['jafix', 'textlint']
const axisMax = 160
</script>

<template>
  <figure
    v-if="frontmatter.benchmark"
    class="home-benchmark-chart"
    :aria-label="`${frontmatter.benchmark.title} (${sample.size})`"
  >
    <dl>
      <div v-for="tool in tools" :key="tool" :class="['chart-row', tool]">
        <dt>{{ tool }}</dt>
        <dd>
          <span class="time">{{ sample[tool] }} <span class="unit">ms</span></span>
          <div class="track" aria-hidden="true">
            <div class="bar" :style="{ width: (sample[tool] / axisMax) * 100 + '%' }"></div>
          </div>
        </dd>
      </div>
    </dl>
    <div class="axis" aria-hidden="true">
      <span>0</span><span>{{ axisMax / 2 }}</span><span>{{ axisMax }} ms</span>
    </div>
  </figure>
</template>

<style>
.VPHero:has(.home-benchmark-chart) .container {
  gap: 32px;
}

.VPHero .image:has(.home-benchmark-chart) {
  order: 3;
  margin: 0;
  min-width: 0;
}

.VPHero .image-container:has(.home-benchmark-chart) {
  width: 100%;
  height: 100%;
  transform: none;
}

@media (min-width: 960px) {
  .VPHero.has-image:has(.home-benchmark-chart) .main {
    width: 54%;
  }
}
</style>

<style scoped>
.home-benchmark-chart {
  width: min(100%, 480px);
  margin-inline: auto;
  padding: clamp(20px, 3vw, 36px);
  border: 1px solid var(--vp-c-divider);
  border-radius: 20px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  text-align: left;
}

.chart-row {
  display: grid;
  grid-template-columns: 1fr auto;
  align-items: baseline;
  row-gap: 8px;
}

.chart-row + .chart-row {
  margin-top: 18px;
}

dt {
  font-size: 16px;
  font-weight: 600;
}

dd {
  display: contents;
}

.time {
  font-size: clamp(28px, 4vw, 44px);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  line-height: 1;
}

.unit {
  color: var(--vp-c-text-2);
  font-size: 15px;
  font-weight: 500;
}

.jafix .time {
  color: var(--vp-c-green-1);
}

.track {
  grid-column: 1 / -1;
  height: 12px;
  overflow: hidden;
  border-radius: 4px;
  background: var(--vp-c-default-soft);
}

.bar {
  height: 100%;
  border-radius: inherit;
  background: var(--vp-c-text-2);
}

.jafix .bar {
  background: var(--vp-c-green-1);
}

.axis {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  margin-top: 8px;
  color: var(--vp-c-text-2);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.axis span:nth-child(2) {
  text-align: center;
}

.axis span:last-child {
  text-align: right;
}
</style>
