<script setup lang="ts">
import { ref, watch } from "vue";
import { NSpace, NInputNumber, NButton, useDialog, useMessage } from "naive-ui";
import { scaleResource } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  kind: string;
  namespace?: string;
  name: string;
  currentReplicas: number;
}>();
const emit = defineEmits<{ scaled: [] }>();

const cluster = useClusterStore();
const dialog = useDialog();
const message = useMessage();
const replicas = ref(props.currentReplicas);

watch(
  () => props.currentReplicas,
  (v) => (replicas.value = v),
);

function confirmScale() {
  dialog.warning({
    title: "Scale?",
    content: `Set replicas for ${props.kind} "${props.name}" to ${replicas.value}?`,
    positiveText: "Scale",
    negativeText: "Cancel",
    onPositiveClick: async () => {
      const contextName = cluster.currentContext;
      if (!contextName) return;
      try {
        await scaleResource(contextName, props.kind, props.namespace, props.name, replicas.value);
        message.success("Scaled");
        emit("scaled");
      } catch (e) {
        message.error(String(e));
      }
    },
  });
}
</script>

<template>
  <n-space align="center" :size="8">
    <n-input-number v-model:value="replicas" :min="0" style="width: 120px" />
    <n-button size="small" @click="confirmScale">Scale</n-button>
  </n-space>
</template>
