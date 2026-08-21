<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute } from "vue-router";
import * as yaml from "js-yaml";
import { NCard, NSpace, NTabs, NTabPane, NText, NDataTable, NAlert, NTag, type DataTableColumns } from "naive-ui";
import YamlViewer from "../components/YamlViewer.vue";
import { getHelmRelease, listHelmReleaseHistory, type HelmReleaseSummary } from "../api/helm";
import { useClusterStore } from "../stores/cluster";

const route = useRoute();
const cluster = useClusterStore();

const namespace = computed(() => String(route.params.namespace));
const name = computed(() => String(route.params.name));

const release = ref<any | null>(null);
const history = ref<HelmReleaseSummary[]>([]);
const selectedRevision = ref<number | undefined>(undefined);
const loading = ref(false);
const error = ref<string | null>(null);

async function load() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  loading.value = true;
  error.value = null;
  try {
    const [rel, hist] = await Promise.all([
      getHelmRelease(contextName, namespace.value, name.value, selectedRevision.value),
      listHelmReleaseHistory(contextName, namespace.value, name.value),
    ]);
    release.value = rel;
    history.value = hist;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
watch([namespace, name, selectedRevision, () => cluster.currentContext], load);

const valuesYaml = computed(() =>
  release.value ? yaml.dump(release.value.config ?? {}, { noRefs: true }) : "",
);
const manifestYaml = computed(() => release.value?.manifest ?? "");

function statusType(status: string): "success" | "warning" | "error" | "default" {
  if (status === "deployed") return "success";
  if (status === "failed") return "error";
  if (status === "pending-install" || status === "pending-upgrade") return "warning";
  return "default";
}

function viewRevision(revision: number) {
  selectedRevision.value = revision;
}

const historyColumns: DataTableColumns<HelmReleaseSummary> = [
  { title: "Revision", key: "revision" },
  { title: "Status", key: "status" },
  { title: "Chart", key: "chart", render: (row) => `${row.chart}-${row.chartVersion}` },
  { title: "Updated", key: "updated" },
  { title: "Description", key: "description" },
];
</script>

<template>
  <n-card :title="`Helm Release: ${name}`">
    <n-space vertical size="large">
      <n-alert v-if="error" type="error" :title="error" closable />
      <n-space align="center">
        <n-text depth="3">Namespace: {{ namespace }}</n-text>
        <n-tag v-if="release" :type="statusType(release.info?.status)" size="small" round>
          {{ release.info?.status }}
        </n-tag>
        <n-text v-if="release" depth="3">
          {{ release.chart?.metadata?.name }}-{{ release.chart?.metadata?.version }} (revision
          {{ release.version }})
        </n-text>
        <n-tag v-if="selectedRevision" closable @close="selectedRevision = undefined">
          viewing revision {{ selectedRevision }}
        </n-tag>
      </n-space>
      <n-tabs type="line">
        <n-tab-pane name="values" tab="Values">
          <YamlViewer :value="valuesYaml" />
        </n-tab-pane>
        <n-tab-pane name="manifest" tab="Manifest">
          <YamlViewer :value="manifestYaml" />
        </n-tab-pane>
        <n-tab-pane name="history" tab="History">
          <n-data-table
            :columns="historyColumns"
            :data="history"
            :loading="loading"
            :bordered="false"
            size="small"
            :row-props="
              (row: HelmReleaseSummary) => ({
                style: 'cursor: pointer',
                onClick: () => viewRevision(row.revision),
              })
            "
          />
        </n-tab-pane>
      </n-tabs>
    </n-space>
  </n-card>
</template>
