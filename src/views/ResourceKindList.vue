<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NCard, NSelect, NSpace, NText, NInput } from "naive-ui";
import ResourceTable from "../components/ResourceTable.vue";
import SavedViewControls from "../components/SavedViewControls.vue";
import { useResourceList } from "../composables/useResourceList";
import { findResourceKind } from "../resourceKinds";
import { looksLikeLabelSelector } from "../labelSelector";

const route = useRoute();
const router = useRouter();

const config = computed(() => findResourceKind(String(route.params.kindRoute)));

const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);
const labelSelector = ref<string>("");
const trimmedFilter = computed(() => labelSelector.value.trim());
const isSelectorSyntax = computed(() => looksLikeLabelSelector(trimmedFilter.value));
// The same input does double duty: `key=value` syntax narrows via the API as a real label
// selector, anything else is a client-side substring search on the resource name.
const effectiveLabelSelector = computed(() =>
  isSelectorSyntax.value ? trimmedFilter.value : undefined,
);
const effectiveSearch = computed(() => (isSelectorSyntax.value ? undefined : trimmedFilter.value));

const { items: namespaces } = useResourceList("Namespace");
const namespaceOptions = computed(() => [
  { label: "All namespaces", value: "" },
  ...namespaces.value.map((ns: any) => ({ label: ns.metadata.name, value: ns.metadata.name })),
]);

// Seed filters from ?ns=&labels= when navigating here (e.g. from the command palette's
// saved-view entries) or when the kind itself changes (this component is reused across
// kind navigations, so a plain onMounted wouldn't fire again).
watch(
  () => route.params.kindRoute,
  () => {
    const ns = route.query.ns;
    const labels = route.query.labels;
    namespace.value = typeof ns === "string" ? ns : "";
    labelSelector.value = typeof labels === "string" ? labels : "";
  },
  { immediate: true },
);

function applySavedView(view: { namespace?: string; labelSelector?: string }) {
  namespace.value = view.namespace ?? "";
  labelSelector.value = view.labelSelector ?? "";
}

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
      <n-space align="center">
        <n-select
          v-if="config.namespaced"
          v-model:value="namespace"
          :options="namespaceOptions"
          style="width: 240px"
          placeholder="All namespaces"
        />
        <n-input
          v-model:value="labelSelector"
          placeholder="Search by name, or label selector (app=foo,tier=bar)"
          style="width: 320px"
          clearable
        />
      </n-space>
      <SavedViewControls
        :kind="config.kind"
        :route="config.route"
        :namespace="effectiveNamespace"
        :label-selector="effectiveLabelSelector"
        @apply="applySavedView"
      />
      <ResourceTable
        :kind="config.kind"
        :namespace="config.namespaced ? effectiveNamespace : undefined"
        :label-selector="effectiveLabelSelector"
        :search="effectiveSearch"
        :show-namespace="config.namespaced"
        :columns="config.columns"
        :row-props="onRowProps"
      />
    </n-space>
  </n-card>
  <n-text v-else depth="3">Unknown resource kind.</n-text>
</template>
