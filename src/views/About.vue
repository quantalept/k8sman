<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NCard, NSpace, NText, NButton, NSpin, NAlert } from "naive-ui";
import { openUrl } from "@tauri-apps/plugin-opener";

const REPO = "https://github.com/quantalept/k8sman";
const version = __APP_VERSION__;

const notices = ref<string>("");
const loading = ref(true);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    const res = await fetch("/THIRD_PARTY_LICENSES.txt");
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    notices.value = await res.text();
  } catch (e) {
    error.value = `Could not load third-party license notices: ${e}`;
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <n-space vertical size="large">
    <n-card title="About k8sman">
      <n-space vertical size="small">
        <n-text>Version {{ version }}</n-text>
        <n-text depth="3">A lightweight Kubernetes desktop client.</n-text>
        <n-text>
          Licensed under the Apache License, Version 2.0. This product bundles third-party
          open-source software; the notices below are also shipped with every build.
        </n-text>
        <n-space>
          <n-button size="small" @click="openUrl(REPO)">Source code</n-button>
          <n-button size="small" @click="openUrl('https://www.apache.org/licenses/LICENSE-2.0')">
            Apache-2.0 license
          </n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-card title="Third-party licenses">
      <n-alert v-if="error" type="error" :title="error" />
      <n-spin v-else :show="loading">
        <pre class="notices">{{ notices }}</pre>
      </n-spin>
    </n-card>
  </n-space>
</template>

<style scoped>
.notices {
  max-height: calc(100vh - 320px);
  overflow: auto;
  margin: 0;
  padding: 12px;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  background: var(--n-color-embedded, rgba(0, 0, 0, 0.2));
  border-radius: 4px;
}
</style>
