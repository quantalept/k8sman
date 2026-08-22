<script setup lang="ts">
import { ref, computed, onMounted, watch, h } from "vue";
import { useRouter } from "vue-router";
import { NCard, NSelect, NSpace, NDataTable, NAlert, NTag, type DataTableColumns } from "naive-ui";
import { listHelmReleases, type HelmReleaseSummary } from "../api/helm";
import { useResourceList } from "../composables/useResourceList";
import { useClusterStore } from "../stores/cluster";
import { PAGE_TABLE_MAX_HEIGHT } from "../layout";

const router = useRouter();
const cluster = useClusterStore();

const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);

const { items: namespaces } = useResourceList("Namespace");
const namespaceOptions = computed(() => [
  { label: "All namespaces", value: "" },
  ...namespaces.value.map((ns: any) => ({ label: ns.metadata.name, value: ns.metadata.name })),
]);

const releases = ref<HelmReleaseSummary[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

async function load() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  loading.value = true;
  error.value = null;
  try {
    releases.value = await listHelmReleases(contextName, effectiveNamespace.value);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
watch([effectiveNamespace, () => cluster.currentContext], load);

function statusType(status: string): "success" | "warning" | "error" | "default" {
  if (status === "deployed") return "success";
  if (status === "failed") return "error";
  if (status === "pending-install" || status === "pending-upgrade") return "warning";
  return "default";
}

const columns: DataTableColumns<HelmReleaseSummary> = [
  { title: "Name", key: "name", sorter: (a, b) => a.name.localeCompare(b.name) },
  { title: "Namespace", key: "namespace", sorter: (a, b) => a.namespace.localeCompare(b.namespace) },
  {
    title: "Chart",
    key: "chart",
    render: (row) => `${row.chart}-${row.chartVersion}`,
    sorter: (a, b) => a.chart.localeCompare(b.chart),
  },
  { title: "App Version", key: "appVersion", sorter: (a, b) => a.appVersion.localeCompare(b.appVersion) },
  {
    title: "Status",
    key: "status",
    render: (row) =>
      h(NTag, { type: statusType(row.status), size: "small", round: true }, { default: () => row.status }),
    sorter: (a, b) => a.status.localeCompare(b.status),
  },
  { title: "Revision", key: "revision", sorter: (a, b) => a.revision - b.revision },
  { title: "Updated", key: "updated", sorter: (a, b) => a.updated.localeCompare(b.updated) },
];
</script>

<template>
  <n-card title="Helm Releases">
    <n-space vertical>
      <n-select
        v-model:value="namespace"
        :options="namespaceOptions"
        style="width: 240px"
        placeholder="All namespaces"
      />
      <n-alert v-if="error" type="error" :title="error" closable />
      <n-data-table
        :columns="columns"
        :data="releases"
        :loading="loading"
        :max-height="PAGE_TABLE_MAX_HEIGHT"
        :bordered="false"
        size="small"
        :row-props="
          (row: HelmReleaseSummary) => ({
            style: 'cursor: pointer',
            onClick: () => router.push(`/helm/${row.namespace}/${row.name}`),
          })
        "
      />
    </n-space>
  </n-card>
</template>
