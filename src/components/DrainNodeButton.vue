<script setup lang="ts">
import { ref } from "vue";
import { NButton, NModal, NCard, NSpace, NText, NList, NListItem, useDialog, useMessage } from "naive-ui";
import { startNodeDrain, type NodeDrainEvent } from "../api/nodes";
import { listResources } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{ name: string }>();

const cluster = useClusterStore();
const dialog = useDialog();
const message = useMessage();

const showProgress = ref(false);
const events = ref<NodeDrainEvent[]>([]);
const done = ref(false);
let stop: (() => void) | null = null;

function describe(e: NodeDrainEvent): string {
  switch (e.state) {
    case "skipped-daemonset":
      return `Skipped ${e.pod} (DaemonSet pod)`;
    case "skipped-mirror":
      return `Skipped ${e.pod} (static/mirror pod)`;
    case "evicting":
      return `Evicting ${e.pod}...`;
    case "evicted":
      return `Evicted ${e.pod}`;
    case "error":
      return `Failed to evict ${e.pod}: ${e.message}`;
    case "done":
      return "Drain complete";
  }
}

async function confirmDrain() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  let evictable = 0;
  try {
    const pods = await listResources(contextName, "Pod", undefined, `spec.nodeName=${props.name}`);
    evictable = pods.filter(
      (p: any) => !(p.metadata?.ownerReferences ?? []).some((r: any) => r.kind === "DaemonSet"),
    ).length;
  } catch {
    // Confirm dialog just won't show a count; the drain stream surfaces real errors.
  }

  dialog.warning({
    title: "Drain node?",
    content: `This will evict ${evictable} pod(s) from node "${props.name}" (DaemonSet and static pods are skipped). This is a single pass, not a retry-until-drained loop - pods blocked by a PodDisruptionBudget will show an error.`,
    positiveText: "Drain",
    negativeText: "Cancel",
    onPositiveClick: startDrain,
  });
}

async function startDrain() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  events.value = [];
  done.value = false;
  showProgress.value = true;
  try {
    stop = await startNodeDrain(contextName, props.name, (event) => {
      events.value = [...events.value, event];
      if (event.state === "done") done.value = true;
    });
  } catch (e) {
    message.error(String(e));
    showProgress.value = false;
  }
}

function closeProgress() {
  stop?.();
  stop = null;
  showProgress.value = false;
}
</script>

<template>
  <n-button size="small" @click="confirmDrain">Drain</n-button>
  <n-modal v-model:show="showProgress" :mask-closable="false">
    <n-card style="width: 480px" title="Draining node" :bordered="false" size="huge" role="dialog">
      <n-space vertical>
        <n-list bordered size="small" style="max-height: 320px; overflow-y: auto">
          <n-list-item v-for="(e, i) in events" :key="i">
            <n-text :type="e.state === 'error' ? 'error' : undefined">{{ describe(e) }}</n-text>
          </n-list-item>
        </n-list>
        <n-space justify="end">
          <n-button size="small" @click="closeProgress">{{ done ? "Close" : "Stop" }}</n-button>
        </n-space>
      </n-space>
    </n-card>
  </n-modal>
</template>
