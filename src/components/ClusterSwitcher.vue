<script setup lang="ts">
import { computed, onMounted } from "vue";
import { NTag, NSpace, NSpin, NButton, NIcon, NPopselect, NText, type SelectOption } from "naive-ui";
import { AddOutline } from "@vicons/ionicons5";
import { useClusterStore } from "../stores/cluster";

const store = useClusterStore();

onMounted(async () => {
  await store.loadContexts();
  await store.restoreTabs();
});

const addableOptions = computed<SelectOption[]>(() =>
  store.contexts
    .filter((c) => !store.connectedContexts.includes(c.name))
    .map((c) => ({ label: c.name, value: c.name })),
);

function handleTabClick(name: string) {
  store.activateTab(name);
}

function handleTabClose(name: string) {
  store.closeTab(name);
}

async function handleAdd(value: string) {
  await store.connectTab(value);
}
</script>

<template>
  <div class="cluster-switcher">
    <n-space :size="6" align="center">
      <n-tag
        v-for="name in store.connectedContexts"
        :key="name"
        :type="name === store.currentContext ? 'success' : 'default'"
        :bordered="name === store.currentContext"
        round
        closable
        size="small"
        class="cluster-tab"
        @click="handleTabClick(name)"
        @close.stop="handleTabClose(name)"
      >
        {{ name }}
      </n-tag>
      <n-text v-if="store.connectedContexts.length === 0" depth="3" style="font-size: 13px">
        No cluster connected
      </n-text>
    </n-space>
    <n-popselect :options="addableOptions" trigger="click" @update:value="handleAdd">
      <n-button size="small" circle quaternary :disabled="addableOptions.length === 0">
        <template #icon>
          <n-icon><AddOutline /></n-icon>
        </template>
      </n-button>
    </n-popselect>
    <n-spin v-if="store.connecting" size="small" />
  </div>
</template>

<style scoped>
.cluster-switcher {
  display: flex;
  align-items: center;
  gap: 8px;
}

.cluster-tab {
  cursor: pointer;
}
</style>
