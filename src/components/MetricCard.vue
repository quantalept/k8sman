<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
import UPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { NCard, NText } from "naive-ui";

const props = defineProps<{
  title: string;
  /** Big headline value, e.g. "85%". */
  value: string;
  color: string;
  /** [timestamps (seconds), values] arrays, same length. */
  series: [number[], number[]];
}>();

const containerEl = ref<HTMLDivElement | null>(null);
let plot: UPlot | null = null;
let resizeObserver: ResizeObserver | null = null;

function buildOptions(width: number): UPlot.Options {
  return {
    width,
    height: 100,
    padding: [4, 4, 4, 4],
    cursor: { show: false },
    legend: { show: false },
    axes: [{ show: false }, { show: false }],
    series: [
      {},
      {
        stroke: props.color,
        width: 2,
        fill: `${props.color}22`,
        points: { show: false },
      },
    ],
  };
}

function render() {
  if (!containerEl.value) return;
  const width = containerEl.value.clientWidth || 300;
  if (!plot) {
    plot = new UPlot(buildOptions(width), props.series, containerEl.value);
  } else {
    plot.setSize({ width, height: 100 });
    plot.setData(props.series);
  }
}

onMounted(() => {
  render();
  if (containerEl.value) {
    resizeObserver = new ResizeObserver(() => render());
    resizeObserver.observe(containerEl.value);
  }
});

watch(() => props.series, render, { deep: true });

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  plot?.destroy();
});
</script>

<template>
  <n-card :title="title">
    <n-text :style="{ fontSize: '32px', fontWeight: 700, color }">{{ value }}</n-text>
    <div ref="containerEl" style="width: 100%; margin-top: 8px" />
  </n-card>
</template>
