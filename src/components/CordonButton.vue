<script setup lang="ts">
import { NButton, useDialog, useMessage } from "naive-ui";
import { cordonNode } from "../api/nodes";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  name: string;
  unschedulable: boolean;
}>();
const emit = defineEmits<{ changed: [] }>();

const cluster = useClusterStore();
const dialog = useDialog();
const message = useMessage();

function confirm() {
  const cordoning = !props.unschedulable;
  dialog.warning({
    title: cordoning ? "Cordon node?" : "Uncordon node?",
    content: cordoning
      ? `This marks node "${props.name}" unschedulable - no new pods will be placed on it. Existing pods keep running.`
      : `This marks node "${props.name}" schedulable again.`,
    positiveText: cordoning ? "Cordon" : "Uncordon",
    negativeText: "Cancel",
    onPositiveClick: async () => {
      const contextName = cluster.currentContext;
      if (!contextName) return;
      try {
        await cordonNode(contextName, props.name, cordoning);
        message.success(cordoning ? "Node cordoned" : "Node uncordoned");
        emit("changed");
      } catch (e) {
        message.error(String(e));
      }
    },
  });
}
</script>

<template>
  <n-button size="small" @click="confirm">{{ unschedulable ? "Uncordon" : "Cordon" }}</n-button>
</template>
