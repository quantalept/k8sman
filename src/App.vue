<script setup lang="ts">
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
</script>

<template>
  <n-config-provider :theme="darkTheme">
    <n-global-style />
    <n-message-provider>
      <n-dialog-provider>
        <n-layout style="height: 100vh">
          <n-layout-header class="app-header" bordered>
            <n-text strong class="app-title">k8sman</n-text>
            <div class="app-header-spacer" />
            <ClusterSwitcher />
          </n-layout-header>
          <n-layout has-sider style="height: calc(100vh - 48px)">
            <n-layout-sider bordered :width="220" content-style="padding: 8px 0;">
              <Sidebar />
            </n-layout-sider>
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
</style>
