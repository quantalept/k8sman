<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  darkTheme,
  NConfigProvider,
  NGlobalStyle,
  NLayout,
  NLayoutSider,
  NLayoutHeader,
  NLayoutContent,
  NText,
  NDialogProvider,
  NMessageProvider,
} from "naive-ui";
import Sidebar from "./components/Sidebar.vue";
import ClusterSwitcher from "./components/ClusterSwitcher.vue";
import CommandPalette from "./components/CommandPalette.vue";
import { registerCommandPaletteShortcut } from "./composables/useCommandPalette";
import { getUiState, setUiState } from "./api/uiState";

registerCommandPaletteShortcut();

// naive-ui's NLayoutSider has no built-in resize handle in the installed version, so this
// drags a thin handle at the sider's right edge and persists the chosen width.
const MIN_WIDTH = 180;
const MAX_WIDTH = 480;
const DEFAULT_WIDTH = 220;
const KEY_SIDEBAR_WIDTH = "sidebarWidth";

const siderWidth = ref(DEFAULT_WIDTH);
let resizing = false;

onMounted(async () => {
  const saved = await getUiState<number>(KEY_SIDEBAR_WIDTH);
  if (saved) siderWidth.value = saved;
});

function startResize(e: MouseEvent) {
  e.preventDefault();
  resizing = true;
  window.addEventListener("mousemove", onResize);
  window.addEventListener("mouseup", stopResize);
}

function onResize(e: MouseEvent) {
  if (!resizing) return;
  siderWidth.value = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, e.clientX));
}

function stopResize() {
  if (!resizing) return;
  resizing = false;
  window.removeEventListener("mousemove", onResize);
  window.removeEventListener("mouseup", stopResize);
  void setUiState(KEY_SIDEBAR_WIDTH, siderWidth.value);
}
</script>

<template>
  <n-config-provider :theme="darkTheme">
    <n-global-style />
    <n-message-provider>
      <n-dialog-provider>
        <CommandPalette />
        <n-layout style="height: 100vh">
          <n-layout-header class="app-header" bordered>
            <n-text strong class="app-title">k8sman</n-text>
            <div class="app-header-spacer" />
            <ClusterSwitcher />
          </n-layout-header>
          <n-layout has-sider class="app-body" style="height: calc(100vh - 48px)">
            <n-layout-sider bordered :width="siderWidth" content-style="padding: 8px 0;">
              <Sidebar />
            </n-layout-sider>
            <div class="sidebar-resize-handle" :style="{ left: siderWidth + 'px' }" @mousedown="startResize" />
            <n-layout-content content-style="padding: 24px;">
              <router-view />
            </n-layout-content>
          </n-layout>
        </n-layout>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
html,
body,
#app {
  margin: 0;
  height: 100%;
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
}

.app-header {
  height: 48px;
  display: flex;
  align-items: center;
  padding: 0 16px;
}

.app-title {
  font-size: 18px;
}

.app-header-spacer {
  flex: 1;
}

.app-body {
  position: relative;
}

.sidebar-resize-handle {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 6px;
  margin-left: -3px;
  cursor: col-resize;
  z-index: 10;
}

.sidebar-resize-handle:hover {
  background: rgba(255, 255, 255, 0.12);
}
</style>
