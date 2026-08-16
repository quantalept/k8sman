<script setup lang="ts">
import { computed, onMounted } from "vue";
import { NSelect, NTag, NSpin, type SelectOption } from "naive-ui";
import { useClusterStore } from "../stores/cluster";

const store = useClusterStore();

onMounted(async () => {
  await store.loadContexts();
  await store.restoreLastContext();
});

const options = computed<SelectOption[]>(() =>
  store.contexts.map((c) => ({
    label: c.name,
    value: c.name,
  })),
);

async function handleUpdate(value: string) {
  await store.switchContext(value);
}
</script>

<template>
  <div class="cluster-switcher">
    <n-select
      :value="store.currentContext ?? undefined"
      :options="options"
      placeholder="Select a context"
      size="small"
      style="width: 220px"
      :loading="store.connecting"
      @update:value="handleUpdate"
    />
    <n-spin v-if="store.connecting" size="small" />
    <n-tag v-else-if="store.connected" type="success" size="small" round>connected</n-tag>
    <n-tag v-else type="default" size="small" round>not connected</n-tag>
  </div>
</template>

<style scoped>
.cluster-switcher {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
