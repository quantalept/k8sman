<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { NSpace, NInput, NButton, NProgress, NAlert, NText, NDivider } from "naive-ui";
import { cpToPod, cpFromPod, type CpProgress } from "../api/cp";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  namespace: string;
  pod: string;
  container?: string;
}>();

const cluster = useClusterStore();

const uploadLocalPath = ref<string | null>(null);
const uploadRemoteDir = ref("/tmp");
const uploadProgress = ref<CpProgress | null>(null);
const uploading = ref(false);
const uploadError = ref<string | null>(null);

const downloadRemotePath = ref("");
const downloadLocalDir = ref<string | null>(null);
const downloadProgress = ref<CpProgress | null>(null);
const downloading = ref(false);
const downloadError = ref<string | null>(null);

async function pickUploadFile() {
  const selected = await open({ multiple: false, title: "Select a file or folder to upload" });
  if (typeof selected === "string") uploadLocalPath.value = selected;
}

async function pickDownloadDir() {
  const selected = await open({ directory: true, title: "Select a destination folder" });
  if (typeof selected === "string") downloadLocalDir.value = selected;
}

async function doUpload() {
  const contextName = cluster.currentContext;
  if (!contextName || !uploadLocalPath.value || !uploadRemoteDir.value) return;
  uploading.value = true;
  uploadError.value = null;
  uploadProgress.value = null;
  try {
    await cpToPod(
      { contextName, namespace: props.namespace, pod: props.pod, container: props.container },
      uploadLocalPath.value,
      uploadRemoteDir.value,
      (p) => (uploadProgress.value = p),
    );
  } catch (e) {
    uploadError.value = String(e);
  } finally {
    uploading.value = false;
  }
}

async function doDownload() {
  const contextName = cluster.currentContext;
  if (!contextName || !downloadRemotePath.value || !downloadLocalDir.value) return;
  downloading.value = true;
  downloadError.value = null;
  downloadProgress.value = null;
  try {
    await cpFromPod(
      { contextName, namespace: props.namespace, pod: props.pod, container: props.container },
      downloadRemotePath.value,
      downloadLocalDir.value,
      (p) => (downloadProgress.value = p),
    );
  } catch (e) {
    downloadError.value = String(e);
  } finally {
    downloading.value = false;
  }
}
</script>

<template>
  <n-space vertical size="large">
    <div>
      <n-text strong>Upload to pod</n-text>
      <n-space vertical style="margin-top: 8px">
        <n-space align="center">
          <n-button size="small" @click="pickUploadFile">Choose file/folder…</n-button>
          <n-text depth="3">{{ uploadLocalPath ?? "nothing selected" }}</n-text>
        </n-space>
        <n-input v-model:value="uploadRemoteDir" placeholder="Remote destination directory" style="width: 320px" />
        <n-button type="primary" size="small" :loading="uploading" :disabled="!uploadLocalPath" @click="doUpload">
          Upload
        </n-button>
        <n-progress
          v-if="uploadProgress"
          type="line"
          :percentage="Math.round((uploadProgress.sent / Math.max(uploadProgress.total, 1)) * 100)"
        />
        <n-alert v-if="uploadError" type="error" :title="uploadError" closable />
      </n-space>
    </div>

    <n-divider style="margin: 0" />

    <div>
      <n-text strong>Download from pod</n-text>
      <n-space vertical style="margin-top: 8px">
        <n-input v-model:value="downloadRemotePath" placeholder="Remote file or folder path" style="width: 320px" />
        <n-space align="center">
          <n-button size="small" @click="pickDownloadDir">Choose destination folder…</n-button>
          <n-text depth="3">{{ downloadLocalDir ?? "nothing selected" }}</n-text>
        </n-space>
        <n-button
          type="primary"
          size="small"
          :loading="downloading"
          :disabled="!downloadRemotePath || !downloadLocalDir"
          @click="doDownload"
        >
          Download
        </n-button>
        <n-text v-if="downloadProgress" depth="3">{{ downloadProgress.sent }} bytes received</n-text>
        <n-alert v-if="downloadError" type="error" :title="downloadError" closable />
      </n-space>
    </div>
  </n-space>
</template>
