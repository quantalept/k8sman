<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { NDataTable, NAlert, NText, type DataTableColumns } from "naive-ui";
import { getSubjectRules, type BindingRules } from "../api/rbac";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  namespace?: string;
  name: string;
}>();

const cluster = useClusterStore();
const bindings = ref<BindingRules[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

interface Row {
  bindingKind: string;
  bindingName: string;
  roleKind: string;
  roleName: string;
  apiGroups: string;
  resources: string;
  verbs: string;
  resourceNames: string;
}

const rows = ref<Row[]>([]);

async function load() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  loading.value = true;
  error.value = null;
  try {
    bindings.value = await getSubjectRules(contextName, "ServiceAccount", props.namespace, props.name);
    rows.value = bindings.value.flatMap((b) =>
      b.rules.map((r) => ({
        bindingKind: b.bindingKind,
        bindingName: b.bindingName,
        roleKind: b.roleKind,
        roleName: b.roleName,
        apiGroups: r.apiGroups.length ? r.apiGroups.map((g) => g || "core").join(", ") : "core",
        resources: r.resources.join(", ") || (r.nonResourceUrls.join(", ") || "-"),
        verbs: r.verbs.join(", "),
        resourceNames: r.resourceNames.join(", ") || "*",
      })),
    );
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
watch([() => props.namespace, () => props.name, () => cluster.currentContext], load);

const columns: DataTableColumns<Row> = [
  {
    title: "Via",
    key: "via",
    render: (row) => `${row.bindingKind}/${row.bindingName}`,
    sorter: (a, b) => a.bindingName.localeCompare(b.bindingName),
  },
  {
    title: "Role",
    key: "role",
    render: (row) => `${row.roleKind}/${row.roleName}`,
    sorter: (a, b) => a.roleName.localeCompare(b.roleName),
  },
  { title: "API Group", key: "apiGroups", sorter: (a, b) => a.apiGroups.localeCompare(b.apiGroups) },
  { title: "Resources", key: "resources", sorter: (a, b) => a.resources.localeCompare(b.resources) },
  { title: "Verbs", key: "verbs" },
  { title: "Resource Names", key: "resourceNames" },
];
</script>

<template>
  <n-alert v-if="error" type="error" :title="error" closable style="margin-bottom: 12px" />
  <n-data-table :columns="columns" :data="rows" :loading="loading" :bordered="false" size="small" />
  <n-text v-if="!loading && rows.length === 0 && !error" depth="3">
    No RoleBindings or ClusterRoleBindings reference this ServiceAccount.
  </n-text>
</template>
