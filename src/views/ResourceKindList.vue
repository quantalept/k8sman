<script setup lang="ts">
import { ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NCard, NSelect, NSpace, NText } from "naive-ui";
import ResourceTable from "../components/ResourceTable.vue";
import { useResourceList } from "../composables/useResourceList";
import { findResourceKind } from "../resourceKinds";

const route = useRoute();
const router = useRouter();

const config = computed(() => findResourceKind(String(route.params.kindRoute)));

const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);

const { items: namespaces } = useResourceList("Namespace");
const namespaceOptions = computed(() => [
  { label: "All namespaces", value: "" },
  ...namespaces.value.map((ns: any) => ({ label: ns.metadata.name, value: ns.metadata.name })),
]);

function onRowProps(row: any) {
  return {
    style: "cursor: pointer",
    onClick: () => {
      const query = row.metadata?.namespace ? { ns: row.metadata.namespace } : {};
      router.push({ path: `/resources/${config.value?.kind}/${row.metadata.name}`, query });
    },
  };
}
</script>

<template>
  <n-card v-if="config" :title="config.label">
    <n-space vertical>
      <n-select
        v-if="config.namespaced"
        v-model:value="namespace"
        :options="namespaceOptions"
        style="width: 240px"
        placeholder="All namespaces"
      />
      <ResourceTable
        :kind="config.kind"
        :namespace="config.namespaced ? effectiveNamespace : undefined"
        :show-namespace="config.namespaced"
        :columns="config.columns"
        :row-props="onRowProps"
      />
    </n-space>
  </n-card>
  <n-text v-else depth="3">Unknown resource kind.</n-text>
</template>
