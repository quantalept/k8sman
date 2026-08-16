<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
// Import Monaco's core editor API directly (not the `monaco-editor` barrel), plus just the
// YAML language registration - the barrel pulls in every bundled language (Perl, SQL, a full
// TypeScript language service, etc.), which is unnecessary bloat for a YAML-only editor.
import * as monaco from "monaco-editor/editor/editor.api";
import "monaco-editor/languages/definitions/yaml/register";
import editorWorker from "monaco-editor/editor/editor.worker?worker";
import * as yaml from "js-yaml";
import { NSpace, NButton, NAlert, useDialog } from "naive-ui";
import { getResource, applyResource } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

// Monaco needs a worker for basic editing services (tokenization, etc). We only use plain
// YAML syntax highlighting, no language server, so one generic worker covers every label.
(self as any).MonacoEnvironment = {
  getWorker() {
    return new editorWorker();
  },
};

const props = defineProps<{
  kind: string;
  namespace?: string;
  name: string;
}>();

const cluster = useClusterStore();
const containerEl = ref<HTMLDivElement | null>(null);
const editing = ref(false);
const loading = ref(true);
const saving = ref(false);
const error = ref<string | null>(null);
const dialog = useDialog();

let editor: monaco.editor.IStandaloneCodeEditor | null = null;
let originalYaml = "";

async function load() {
  const contextName = cluster.currentContext;
  if (!contextName || !editor) return;
  loading.value = true;
  error.value = null;
  try {
    const obj = await getResource(contextName, props.kind, props.namespace, props.name);
    originalYaml = yaml.dump(obj, { noRefs: true });
    editor.setValue(originalYaml);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  if (!containerEl.value) return;
  editor = monaco.editor.create(containerEl.value, {
    value: "",
    language: "yaml",
    theme: "vs-dark",
    automaticLayout: true,
    readOnly: !editing.value,
    minimap: { enabled: false },
    fontSize: 13,
  });
  load();
});

watch(() => [props.kind, props.namespace, props.name, cluster.currentContext], load);
watch(editing, (value) => editor?.updateOptions({ readOnly: !value }));

function cancel() {
  editor?.setValue(originalYaml);
  editing.value = false;
}

function save() {
  const value = editor?.getValue() ?? "";
  dialog.warning({
    title: "Apply changes?",
    content: `This will apply your edits to ${props.kind} "${props.name}" on the connected cluster.`,
    positiveText: "Apply",
    negativeText: "Cancel",
    onPositiveClick: async () => {
      const contextName = cluster.currentContext;
      if (!contextName) return;
      saving.value = true;
      error.value = null;
      try {
        const applied = await applyResource(contextName, value);
        originalYaml = yaml.dump(applied, { noRefs: true });
        editor?.setValue(originalYaml);
        editing.value = false;
      } catch (e) {
        error.value = String(e);
      } finally {
        saving.value = false;
      }
    },
  });
}

onBeforeUnmount(() => {
  editor?.dispose();
});
</script>

<template>
  <n-space vertical>
    <n-alert v-if="error" type="error" :title="error" closable />
    <n-space>
      <n-button v-if="!editing" size="small" :loading="loading" @click="editing = true">Edit</n-button>
      <template v-else>
        <n-button size="small" type="primary" :loading="saving" @click="save">Save</n-button>
        <n-button size="small" :disabled="saving" @click="cancel">Cancel</n-button>
      </template>
    </n-space>
    <div ref="containerEl" style="height: 500px; border-radius: 6px; overflow: hidden" />
  </n-space>
</template>
