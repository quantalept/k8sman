<script setup lang="ts">
import { ref, computed } from "vue";
import { NSpace, NInput, NSelect, NButton, NText, useMessage } from "naive-ui";
import ExecTerminal from "./ExecTerminal.vue";
import { addEphemeralContainer } from "../api/debug";
import { getResource } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  namespace: string;
  pod: string;
  containerOptions: { label: string; value: string }[];
}>();

const cluster = useClusterStore();
const message = useMessage();

const image = ref("busybox:1.36");
const targetContainer = ref<string | undefined>(undefined);
const starting = ref(false);
const activeName = ref<string | null>(null);

async function start() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  starting.value = true;
  const containerName = `debug-${Date.now().toString(36)}`;
  try {
    await addEphemeralContainer(
      contextName,
      props.namespace,
      props.pod,
      containerName,
      image.value,
      targetContainer.value,
    );
    await waitForRunning(contextName, containerName);
    activeName.value = containerName;
  } catch (e) {
    message.error(String(e));
  } finally {
    starting.value = false;
  }
}

async function waitForRunning(contextName: string, containerName: string) {
  for (let i = 0; i < 60; i++) {
    const obj = await getResource(contextName, "Pod", props.namespace, props.pod);
    const status = (obj.status?.ephemeralContainerStatuses ?? []).find(
      (s: any) => s.name === containerName,
    );
    if (status?.state?.running) return;
    if (status?.state?.terminated) throw new Error("ephemeral container exited before it became ready");
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error("timed out waiting for ephemeral container to start");
}

const targetOptions = computed(() => props.containerOptions);
</script>

<template>
  <n-space vertical size="large">
    <template v-if="!activeName">
      <n-text depth="3">
        Attaches a new ephemeral debug container to this running pod. Ephemeral containers cannot
        be removed once attached - the pod keeps it until it's deleted.
      </n-text>
      <n-space align="center" :size="8">
        <n-input v-model:value="image" placeholder="Debug image" style="width: 220px" />
        <n-select
          v-model:value="targetContainer"
          :options="targetOptions"
          placeholder="Target container (optional)"
          clearable
          style="width: 220px"
        />
        <n-button size="small" :loading="starting" @click="start">Attach</n-button>
      </n-space>
    </template>
    <ExecTerminal
      v-else
      :namespace="namespace"
      :pod="pod"
      :container="activeName"
      :command="['/bin/sh']"
    />
  </n-space>
</template>
