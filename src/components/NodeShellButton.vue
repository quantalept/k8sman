<script setup lang="ts">
import { ref } from "vue";
import { NButton, NModal, NCard, NSpace, NInput, NText, useMessage } from "naive-ui";
import ExecTerminal from "./ExecTerminal.vue";
import { createNodeDebugPod } from "../api/debug";
import { getResource, deleteResource } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{ name: string }>();

const cluster = useClusterStore();
const message = useMessage();

const showForm = ref(false);
const showTerminal = ref(false);
const image = ref("busybox:1.36");
const starting = ref(false);
const pod = ref<{ namespace: string; name: string } | null>(null);

function openForm() {
  showForm.value = true;
}

async function start() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  starting.value = true;
  try {
    const ref_ = await createNodeDebugPod(contextName, props.name, image.value || undefined);
    await waitForRunning(contextName, ref_.namespace, ref_.name);
    pod.value = ref_;
    showForm.value = false;
    showTerminal.value = true;
  } catch (e) {
    message.error(String(e));
  } finally {
    starting.value = false;
  }
}

async function waitForRunning(contextName: string, namespace: string, name: string) {
  for (let i = 0; i < 60; i++) {
    const obj = await getResource(contextName, "Pod", namespace, name);
    if (obj.status?.phase === "Running") return;
    if (obj.status?.phase === "Failed") throw new Error("debug pod failed to start");
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error("timed out waiting for debug pod to start");
}

async function closeTerminal() {
  showTerminal.value = false;
  const contextName = cluster.currentContext;
  if (contextName && pod.value) {
    try {
      await deleteResource(contextName, "Pod", pod.value.namespace, pod.value.name);
    } catch (e) {
      message.error(`Failed to clean up debug pod: ${e}`);
    }
  }
  pod.value = null;
}
</script>

<template>
  <n-button size="small" @click="openForm">Node Shell</n-button>

  <n-modal v-model:show="showForm" :mask-closable="!starting">
    <n-card style="width: 420px" title="Open node shell" :bordered="false" size="huge" role="dialog">
      <n-space vertical>
        <n-text depth="3">
          Creates a privileged pod pinned to node "{{ name }}" with the host filesystem mounted at
          /host (in namespace kube-system). Deleted automatically when you close the shell.
        </n-text>
        <n-input v-model:value="image" placeholder="Debug image" />
        <n-space justify="end">
          <n-button size="small" :loading="starting" @click="start">Start</n-button>
        </n-space>
      </n-space>
    </n-card>
  </n-modal>

  <n-modal v-model:show="showTerminal" :mask-closable="false">
    <n-card
      style="width: 720px"
      :title="`Shell on ${name}`"
      :bordered="false"
      size="huge"
      role="dialog"
      closable
      @close="closeTerminal"
    >
      <ExecTerminal
        v-if="pod"
        :namespace="pod.namespace"
        :pod="pod.name"
        container="debug"
        :command="['/bin/sh']"
      />
    </n-card>
  </n-modal>
</template>
