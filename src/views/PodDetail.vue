<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute } from "vue-router";
import { NCard, NSelect, NSpace, NText, NSwitch, NTabs, NTabPane } from "naive-ui";
import LogViewer from "../components/LogViewer.vue";
import ExecTerminal from "../components/ExecTerminal.vue";
import EphemeralDebugPanel from "../components/EphemeralDebugPanel.vue";
import PortForwardManager from "../components/PortForwardManager.vue";
import FileCopyManager from "../components/FileCopyManager.vue";
import PodMetricsPanel from "../components/PodMetricsPanel.vue";
import ResourceYamlEditor from "../components/ResourceYamlEditor.vue";
import ResourceEventsTable from "../components/ResourceEventsTable.vue";
import DeleteResourceButton from "../components/DeleteResourceButton.vue";
import PinButton from "../components/PinButton.vue";
import { listResources } from "../api/resources";
import { useClusterStore } from "../stores/cluster";
import { usePinnedStore } from "../stores/pinned";

const route = useRoute();
const cluster = useClusterStore();
const pinned = usePinnedStore();

const namespace = computed(() => String(route.params.namespace));
const podName = computed(() => String(route.params.name));

const containers = ref<string[]>([]);
const selectedContainer = ref<string | undefined>(undefined);
const follow = ref(true);
const activeTab = ref("logs");

async function loadContainers() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  const pods = await listResources(contextName, "Pod", namespace.value);
  const pod = pods.find((p: any) => p.metadata?.name === podName.value);
  const names: string[] = (pod?.spec?.containers ?? []).map((c: any) => c.name);
  containers.value = names;
  if (!selectedContainer.value || !names.includes(selectedContainer.value)) {
    selectedContainer.value = names[0];
  }
}

onMounted(() => {
  loadContainers();
  pinned.recordVisit({ kind: "Pod", namespace: namespace.value, name: podName.value });
});
watch([namespace, podName, () => cluster.currentContext], loadContainers);
watch([namespace, podName], () =>
  pinned.recordVisit({ kind: "Pod", namespace: namespace.value, name: podName.value }),
);

const containerOptions = computed(() =>
  containers.value.map((name) => ({ label: name, value: name })),
);
</script>

<template>
  <n-card :title="`Pod: ${podName}`">
    <template #header-extra>
      <n-space align="center">
        <PinButton kind="Pod" :namespace="namespace" :name="podName" />
        <DeleteResourceButton kind="Pod" :namespace="namespace" :name="podName" back-path="/pods" />
      </n-space>
    </template>
    <n-space vertical size="large">
      <n-text depth="3">Namespace: {{ namespace }}</n-text>

      <n-select
        v-model:value="selectedContainer"
        :options="containerOptions"
        style="width: 220px"
        placeholder="Select a container"
      />

      <n-tabs v-if="selectedContainer" v-model:value="activeTab" type="line">
        <n-tab-pane name="metrics" tab="Metrics">
          <PodMetricsPanel :namespace="namespace" :pod="podName" />
        </n-tab-pane>
        <n-tab-pane name="logs" tab="Logs">
          <n-space vertical>
            <n-space align="center" :size="4">
              <n-text depth="3">Follow</n-text>
              <n-switch v-model:value="follow" />
            </n-space>
            <LogViewer
              :namespace="namespace"
              :pod="podName"
              :container="selectedContainer"
              :follow="follow"
            />
          </n-space>
        </n-tab-pane>
        <n-tab-pane name="exec" tab="Exec">
          <ExecTerminal
            v-if="activeTab === 'exec'"
            :namespace="namespace"
            :pod="podName"
            :container="selectedContainer"
          />
        </n-tab-pane>
        <n-tab-pane name="debug" tab="Debug">
          <EphemeralDebugPanel
            v-if="activeTab === 'debug'"
            :namespace="namespace"
            :pod="podName"
            :container-options="containerOptions"
          />
        </n-tab-pane>
        <n-tab-pane name="portforward" tab="Port Forward">
          <PortForwardManager :namespace="namespace" :pod="podName" />
        </n-tab-pane>
        <n-tab-pane name="files" tab="Copy Files">
          <FileCopyManager :namespace="namespace" :pod="podName" :container="selectedContainer" />
        </n-tab-pane>
        <n-tab-pane name="yaml" tab="YAML">
          <ResourceYamlEditor kind="Pod" :namespace="namespace" :name="podName" />
        </n-tab-pane>
        <n-tab-pane name="events" tab="Events">
          <ResourceEventsTable :namespace="namespace" :involved-object-name="podName" />
        </n-tab-pane>
      </n-tabs>
      <n-text v-else depth="3">No containers found for this pod.</n-text>
    </n-space>
  </n-card>
</template>
