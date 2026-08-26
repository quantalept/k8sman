<script setup lang="ts">
import { ref, computed, h } from "vue";
import { NSpace, NButton, NInput, NSelect, NDataTable, useMessage, type DataTableColumns } from "naive-ui";
import { taintNode, untaintNode } from "../api/nodes";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  name: string;
  taints: Array<{ key: string; value?: string; effect: string }>;
}>();
const emit = defineEmits<{ changed: [] }>();

const cluster = useClusterStore();
const message = useMessage();

const newKey = ref("");
const newValue = ref("");
const newEffect = ref("NoSchedule");
const effectOptions = [
  { label: "NoSchedule", value: "NoSchedule" },
  { label: "PreferNoSchedule", value: "PreferNoSchedule" },
  { label: "NoExecute", value: "NoExecute" },
];

async function addTaint() {
  const contextName = cluster.currentContext;
  if (!contextName || !newKey.value) return;
  try {
    await taintNode(contextName, props.name, newKey.value, newValue.value || undefined, newEffect.value);
    message.success("Taint added");
    newKey.value = "";
    newValue.value = "";
    emit("changed");
  } catch (e) {
    message.error(String(e));
  }
}

async function removeTaint(key: string, effect: string) {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  try {
    await untaintNode(contextName, props.name, key, effect);
    message.success("Taint removed");
    emit("changed");
  } catch (e) {
    message.error(String(e));
  }
}

const columns = computed<DataTableColumns<any>>(() => [
  { title: "Key", key: "key" },
  { title: "Value", key: "value", render: (row: any) => row.value ?? "-" },
  { title: "Effect", key: "effect" },
  {
    title: "",
    key: "actions",
    render: (row: any) =>
      h(NButton, { size: "small", tertiary: true, onClick: () => removeTaint(row.key, row.effect) }, {
        default: () => "Remove",
      }),
  },
]);
</script>

<template>
  <n-space vertical size="large">
    <n-data-table :columns="columns" :data="taints" :bordered="false" size="small" />
    <n-space align="center" :size="8">
      <n-input v-model:value="newKey" placeholder="key" style="width: 160px" />
      <n-input v-model:value="newValue" placeholder="value (optional)" style="width: 160px" />
      <n-select v-model:value="newEffect" :options="effectOptions" style="width: 160px" />
      <n-button size="small" @click="addTaint" :disabled="!newKey">Add Taint</n-button>
    </n-space>
  </n-space>
</template>
