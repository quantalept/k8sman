<script setup lang="ts">
import { computed, onMounted, watch, ref, h } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  NCard,
  NButton,
  NDataTable,
  NSpace,
  NText,
  NAlert,
  NInput,
  useMessage,
  type DataTableColumns,
} from "naive-ui";
import { useClusterStore, type ContextInfo } from "../stores/cluster";
import { useSavedViewsStore, type SavedView } from "../stores/savedViews";
import { getPrometheusUrl, setPrometheusUrl } from "../api/metrics";

const store = useClusterStore();
const savedViews = useSavedViewsStore();
const message = useMessage();

onMounted(() => {
  store.loadContexts();
  savedViews.load();
});

const prometheusUrls = ref<Record<string, string>>({});

async function loadPrometheusUrls() {
  for (const ctx of store.contexts) {
    if (ctx.name in prometheusUrls.value) continue;
    prometheusUrls.value[ctx.name] = (await getPrometheusUrl(ctx.name)) ?? "";
  }
}

watch(() => store.contexts, loadPrometheusUrls, { immediate: true });

async function savePrometheusUrl(contextName: string) {
  try {
    await setPrometheusUrl(contextName, prometheusUrls.value[contextName] ?? "");
    message.success(`Saved Prometheus URL for ${contextName}`);
  } catch (e) {
    message.error(String(e));
  }
}

async function importKubeconfig() {
  const selected = await open({
    multiple: false,
    title: "Select a kubeconfig file",
  });
  if (typeof selected === "string") {
    await store.addKubeconfig(selected);
  }
}

async function forget(path: string) {
  await store.removeKubeconfig(path);
}

const contextColumns: DataTableColumns<ContextInfo> = [
  { title: "Context", key: "name", sorter: (a, b) => a.name.localeCompare(b.name) },
  { title: "Cluster", key: "cluster", sorter: (a, b) => a.cluster.localeCompare(b.cluster) },
  { title: "Namespace", key: "namespace", render: (row) => row.namespace ?? "default" },
  { title: "Source", key: "source" },
];

const contexts = computed(() => store.contexts);

const savedViewColumns: DataTableColumns<SavedView> = [
  { title: "Name", key: "name", sorter: (a, b) => a.name.localeCompare(b.name) },
  { title: "Kind", key: "kind", sorter: (a, b) => a.kind.localeCompare(b.kind) },
  { title: "Namespace", key: "namespace", render: (row) => row.namespace ?? "all" },
  { title: "Label selector", key: "labelSelector", render: (row) => row.labelSelector ?? "-" },
  {
    title: "",
    key: "actions",
    render: (row) =>
      h(
        NButton,
        { size: "tiny", tertiary: true, type: "error", onClick: () => savedViews.remove(row.id) },
        { default: () => "Delete" },
      ),
  },
];
</script>

<template>
  <n-space vertical size="large">
    <n-card title="Kubeconfig files">
      <n-space vertical>
        <n-alert v-if="store.error" type="error" :title="store.error" closable />
        <n-space v-for="path in store.kubeconfigPaths" :key="path" align="center">
          <n-text>{{ path }}</n-text>
          <n-button size="tiny" tertiary type="error" @click="forget(path)">Forget</n-button>
        </n-space>
        <n-button size="small" @click="importKubeconfig">Import kubeconfig file…</n-button>
      </n-space>
    </n-card>

    <n-card title="Available contexts">
      <n-data-table :columns="contextColumns" :data="contexts" :bordered="false" size="small" />
    </n-card>

    <n-card title="Prometheus">
      <n-space vertical>
        <n-text depth="3">
          Optional. Point at a Prometheus endpoint already reachable from this machine (e.g.
          via your own `kubectl port-forward` or an ingress) to get real historical metrics
          instead of point-in-time metrics-server polling.
        </n-text>
        <n-space v-for="ctx in store.contexts" :key="ctx.name" align="center">
          <n-text style="width: 160px">{{ ctx.name }}</n-text>
          <n-input
            v-model:value="prometheusUrls[ctx.name]"
            placeholder="http://localhost:9090"
            style="width: 320px"
            clearable
          />
          <n-button size="tiny" @click="savePrometheusUrl(ctx.name)">Save</n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-card title="Saved Views">
      <n-data-table
        :columns="savedViewColumns"
        :data="savedViews.views"
        :bordered="false"
        size="small"
      />
    </n-card>
  </n-space>
</template>
