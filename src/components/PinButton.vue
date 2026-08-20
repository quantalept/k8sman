<script setup lang="ts">
import { computed, onMounted } from "vue";
import { NButton, NIcon } from "naive-ui";
import { Bookmark, BookmarkOutline } from "@vicons/ionicons5";
import { usePinnedStore } from "../stores/pinned";

const props = defineProps<{
  kind: string;
  namespace?: string;
  name: string;
}>();

const store = usePinnedStore();

onMounted(() => store.load());

const resource = computed(() => ({ kind: props.kind, namespace: props.namespace, name: props.name }));
const pinned = computed(() => store.isPinned(resource.value));

function toggle() {
  if (pinned.value) {
    store.unpin(resource.value);
  } else {
    store.pin(resource.value);
  }
}
</script>

<template>
  <n-button size="small" tertiary :type="pinned ? 'warning' : 'default'" @click="toggle">
    <template #icon>
      <n-icon><Bookmark v-if="pinned" /><BookmarkOutline v-else /></n-icon>
    </template>
    {{ pinned ? "Pinned" : "Pin" }}
  </n-button>
</template>
