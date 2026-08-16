<script setup lang="ts">
import { ref, computed } from "vue";
import { NCard, NSelect, NSpace } from "naive-ui";
import ResourceTable from "../components/ResourceTable.vue";
import { useResourceList } from "../composables/useResourceList";

const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);

const { items: namespaces } = useResourceList("Namespace");
const namespaceOptions = computed(() => [
  { label: "All namespaces", value: "" },
  ...namespaces.value.map((ns: any) => ({ label: ns.metadata.name, value: ns.metadata.name })),
]);

function ports(row: any): string {
  const p: any[] = row.spec?.ports ?? [];
  return p.map((x) => `${x.port}${x.protocol ? "/" + x.protocol : ""}`).join(", ") || "-";
}

const columns = [
  { title: "Type", key: "type", render: (row: any) => row.spec?.type ?? "ClusterIP" },
  { title: "Cluster IP", key: "clusterIP", render: (row: any) => row.spec?.clusterIP ?? "-" },
  { title: "Ports", key: "ports", render: (row: any) => ports(row) },
];
</script>

<template>
  <n-card title="Services">
    <n-space vertical>
      <n-select
        v-model:value="namespace"
        :options="namespaceOptions"
        style="width: 240px"
        placeholder="All namespaces"
      />
      <ResourceTable kind="Service" :namespace="effectiveNamespace" :columns="columns" />
    </n-space>
  </n-card>
</template>
