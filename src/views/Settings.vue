<script setup lang="ts">
import { computed, onMounted, h } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  NCard,
  NButton,
  NDataTable,
  NSpace,
  NText,
  NAlert,
  type DataTableColumns,
} from "naive-ui";
import { useClusterStore, type ContextInfo } from "../stores/cluster";
import { useSavedViewsStore, type SavedView } from "../stores/savedViews";

const store = useClusterStore();
const savedViews = useSavedViewsStore();

onMounted(() => {
  store.loadContexts();
  savedViews.load();
});

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
  { title: "Context", key: "name" },
  { title: "Cluster", key: "cluster" },
  { title: "Namespace", key: "namespace", render: (row) => row.namespace ?? "default" },
  { title: "Source", key: "source" },
];

const contexts = computed(() => store.contexts);

const savedViewColumns: DataTableColumns<SavedView> = [
  { title: "Name", key: "name" },
  { title: "Kind", key: "kind" },
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
