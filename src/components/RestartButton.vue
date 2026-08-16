<script setup lang="ts">
import { NButton, useDialog, useMessage } from "naive-ui";
import { restartRollout } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  kind: string;
  namespace?: string;
  name: string;
}>();

const cluster = useClusterStore();
const dialog = useDialog();
const message = useMessage();

function confirmRestart() {
  dialog.warning({
    title: "Restart rollout?",
    content: `This will restart all pods managed by ${props.kind} "${props.name}".`,
    positiveText: "Restart",
    negativeText: "Cancel",
    onPositiveClick: async () => {
      const contextName = cluster.currentContext;
      if (!contextName) return;
      try {
        await restartRollout(contextName, props.kind, props.namespace, props.name);
        message.success("Restart triggered");
      } catch (e) {
        message.error(String(e));
      }
    },
  });
}
</script>

<template>
  <n-button size="small" @click="confirmRestart">Restart</n-button>
</template>
