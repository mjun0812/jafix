<script setup>
import { useData, withBase } from 'vitepress'
import { checkResults } from './benchmark-results.js'

const { frontmatter } = useData()
</script>

<template>
  <section v-if="frontmatter.benchmark" class="home-benchmark" aria-labelledby="benchmark-title">
    <h2 id="benchmark-title">{{ frontmatter.benchmark.title }}</h2>
    <div class="table-wrapper">
      <table aria-labelledby="benchmark-title">
        <thead>
          <tr>
            <th scope="col">{{ frontmatter.benchmark.sizeLabel }}</th>
            <th scope="col">jafix</th>
            <th scope="col">textlint</th>
            <th scope="col">{{ frontmatter.benchmark.ratioLabel }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in checkResults" :key="row.size">
            <th scope="row">{{ row.size }}</th>
            <td class="jafix-time">{{ row.jafix }} ms</td>
            <td>{{ row.textlint.toLocaleString('en-US') }} ms</td>
            <td>{{ row.ratio }}{{ frontmatter.benchmark.ratioUnit }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <a :href="withBase(frontmatter.benchmark.link)">{{ frontmatter.benchmark.linkText }}</a>
  </section>
</template>

<style scoped>
.home-benchmark {
  max-width: 1280px;
  margin: 0 auto 48px;
  padding: 0 24px;
}

h2 {
  font-size: 24px;
  font-weight: 600;
  line-height: 1.3;
}

.table-wrapper {
  margin: 16px 0;
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
  font-variant-numeric: tabular-nums;
}

thead {
  background: var(--vp-c-bg-soft);
}

th,
td {
  padding: 8px 4px;
  border-bottom: 1px solid var(--vp-c-divider);
  text-align: right;
  white-space: nowrap;
}

th:first-child {
  text-align: left;
}

tbody th {
  font-weight: normal;
}

.jafix-time {
  font-weight: 600;
}

a {
  color: var(--vp-c-brand-1);
  font-weight: 500;
  text-decoration: underline;
  text-underline-offset: 3px;
}

@media (min-width: 640px) {
  .home-benchmark {
    padding: 0 48px;
  }

  th,
  td {
    padding-inline: 16px;
  }
}

@media (min-width: 960px) {
  .home-benchmark {
    padding: 0 64px;
  }
}
</style>
