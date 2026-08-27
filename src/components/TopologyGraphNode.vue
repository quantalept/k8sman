<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { Handle, Position } from "@vue-flow/core";
import { NIcon, NButton, NTooltip } from "naive-ui";
import { OpenOutline, ExtensionPuzzleOutline } from "@vicons/ionicons5";
import { findResourceKindByKind } from "../resourceKinds";
import { resourcePath } from "../resourcePath";
import type { TopologyGraphNode } from "../composables/useResourceTopology";

const props = defineProps<{ data: TopologyGraphNode }>();
const router = useRouter();

const icon = computed(() => findResourceKindByKind(props.data.kind)?.icon ?? ExtensionPuzzleOutline);

function open(e: MouseEvent) {
  e.stopPropagation();
  router.push(resourcePath(props.data.kind, props.data.namespace, props.data.name));
}
</script>

<template>
  <Handle type="target" :position="Position.Top" />
  <div class="topology-node" :class="{ focal: data.focal }">
    <n-icon size="16" class="topology-node-icon"><component :is="icon" /></n-icon>
    <div class="topology-node-text">
      <n-tooltip trigger="hover">
        <template #trigger>
          <div class="topology-node-name">{{ data.name }}</div>
        </template>
        {{ data.kind }} {{ data.name }}<span v-if="data.namespace"> ({{ data.namespace }})</span>
      </n-tooltip>
      <div class="topology-node-kind">{{ data.kind }}</div>
    </div>
    <n-button size="tiny" quaternary circle @click="open">
      <template #icon><n-icon><OpenOutline /></n-icon></template>
    </n-button>
  </div>
  <Handle type="source" :position="Position.Bottom" />
</template>

<style scoped>
.topology-node {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--n-border-color, #444);
  background: var(--n-color, #1e1e22);
  min-width: 160px;
  max-width: 200px;
}
.topology-node.focal {
  border-color: #e6a23c;
  border-width: 2px;
}
.topology-node-icon {
  flex-shrink: 0;
  opacity: 0.8;
}
.topology-node-text {
  overflow: hidden;
  flex: 1;
}
.topology-node-name {
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.topology-node-kind {
  font-size: 10px;
  opacity: 0.6;
}
</style>
