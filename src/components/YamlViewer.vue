<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
// Same rationale as ResourceYamlEditor.vue: import Monaco's core API directly, not the
// `monaco-editor` barrel (which pulls in ~40 unused languages).
import * as monaco from "monaco-editor/editor/editor.api";
import "monaco-editor/languages/definitions/yaml/register";
import editorWorker from "monaco-editor/editor/editor.worker?worker";

(self as any).MonacoEnvironment = {
  getWorker() {
    return new editorWorker();
  },
};

const props = defineProps<{
  value: string;
  language?: string;
}>();

const containerEl = ref<HTMLDivElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;

onMounted(() => {
  if (!containerEl.value) return;
  editor = monaco.editor.create(containerEl.value, {
    value: props.value,
    language: props.language ?? "yaml",
    theme: "vs-dark",
    automaticLayout: true,
    readOnly: true,
    minimap: { enabled: false },
    fontSize: 13,
  });
});

watch(
  () => props.value,
  (value) => editor?.setValue(value),
);

onBeforeUnmount(() => {
  editor?.dispose();
});
</script>

<template>
  <div ref="containerEl" style="height: 500px; border-radius: 6px; overflow: hidden" />
</template>
