<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { NButton, NSelect, NSpace, NInput, useMessage, type SelectOption } from "naive-ui";
import { useSavedViewsStore } from "../stores/savedViews";

const props = defineProps<{
  kind: string;
  route: string;
  namespace?: string;
  labelSelector?: string;
}>();

const emit = defineEmits<{
  apply: [{ namespace?: string; labelSelector?: string }];
}>();

const store = useSavedViewsStore();
const message = useMessage();
const newViewName = ref("");

onMounted(() => store.load());

const viewsForKind = computed(() => store.views.filter((v) => v.kind === props.kind));
const options = computed<SelectOption[]>(() =>
  viewsForKind.value.map((v) => ({ label: v.name, value: v.id })),
);

async function doSave() {
  const name = newViewName.value.trim();
  if (!name) {
    message.warning("Enter a name for the view first");
    return;
  }
  await store.save({
    name,
    kind: props.kind,
    route: props.route,
    namespace: props.namespace,
    labelSelector: props.labelSelector,
  });
  message.success(`Saved view "${name}"`);
  newViewName.value = "";
}

function applyView(id: string) {
  const view = store.views.find((v) => v.id === id);
  if (!view) return;
  emit("apply", { namespace: view.namespace, labelSelector: view.labelSelector });
}
</script>

<template>
  <n-space align="center">
    <n-select
      v-if="options.length > 0"
      :options="options"
      placeholder="Saved views"
      style="width: 180px"
      size="small"
      @update:value="applyView"
    />
    <n-input v-model:value="newViewName" placeholder="View name" size="small" style="width: 140px" />
    <n-button size="small" @click="doSave">Save view</n-button>
  </n-space>
</template>
