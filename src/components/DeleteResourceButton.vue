<script setup lang="ts">
import { useRouter } from "vue-router";
import { NButton, useDialog, useMessage } from "naive-ui";
import { deleteResource } from "../api/resources";
import { useClusterStore } from "../stores/cluster";
import { findResourceKindByKind } from "../resourceKinds";

const props = defineProps<{
  kind: string;
  namespace?: string;
  name: string;
  /** Where to navigate after a successful delete. Defaults to the kind's list page. */
  backPath?: string;
}>();

const cluster = useClusterStore();
const router = useRouter();
const dialog = useDialog();
const message = useMessage();

function confirmDelete() {
  dialog.error({
    title: "Delete resource?",
    content: `This will permanently delete ${props.kind} "${props.name}"${
      props.namespace ? ` in namespace "${props.namespace}"` : ""
    }. This cannot be undone.`,
    positiveText: "Delete",
    negativeText: "Cancel",
    onPositiveClick: async () => {
      const contextName = cluster.currentContext;
      if (!contextName) return;
      try {
        await deleteResource(contextName, props.kind, props.namespace, props.name);
        message.success(`Deleted ${props.kind} "${props.name}"`);
        const fallback = findResourceKindByKind(props.kind);
        router.push(props.backPath ?? (fallback ? `/${fallback.route}` : "/dashboard"));
      } catch (e) {
        message.error(String(e));
      }
    },
  });
}
</script>

<template>
  <n-button size="small" type="error" tertiary @click="confirmDelete">Delete</n-button>
</template>
