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

const columns = [
  {
    title: "Keys",
    key: "keys",
    render: (row: any) => Object.keys(row.data ?? {}).length,
  },
];
</script>

<template>
  <n-card title="ConfigMaps">
    <n-space vertical>
      <n-select
        v-model:value="namespace"
        :options="namespaceOptions"
        style="width: 240px"
        placeholder="All namespaces"
      />
      <ResourceTable kind="ConfigMap" :namespace="effectiveNamespace" :columns="columns" />
    </n-space>
  </n-card>
</template>
