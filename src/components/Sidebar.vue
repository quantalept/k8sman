<script setup lang="ts">
import { h, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NIcon, NMenu, type MenuOption } from "naive-ui";
import {
  GridOutline,
  FolderOutline,
  ServerOutline,
  CubeOutline,
  GitNetworkOutline,
  DocumentTextOutline,
  LockClosedOutline,
  SaveOutline,
  SwapHorizontalOutline,
  SettingsOutline,
} from "@vicons/ionicons5";

const route = useRoute();
const router = useRouter();

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) });
}

const menuOptions: MenuOption[] = [
  { label: "Dashboard", key: "/dashboard", icon: renderIcon(GridOutline) },
  { label: "Namespaces", key: "/namespaces", icon: renderIcon(FolderOutline) },
  { label: "Nodes", key: "/nodes", icon: renderIcon(ServerOutline) },
  { label: "Pods", key: "/pods", icon: renderIcon(CubeOutline) },
  { label: "Services", key: "/services", icon: renderIcon(GitNetworkOutline) },
  { label: "ConfigMaps", key: "/configmaps", icon: renderIcon(DocumentTextOutline) },
  { label: "Secrets", key: "/secrets", icon: renderIcon(LockClosedOutline) },
  { label: "Storage", key: "/storage", icon: renderIcon(SaveOutline) },
  { label: "Port Forwards", key: "/portforwards", icon: renderIcon(SwapHorizontalOutline) },
  { label: "Settings", key: "/settings", icon: renderIcon(SettingsOutline) },
];

const activeKey = computed(() => "/" + route.path.split("/")[1]);

function handleUpdate(key: string) {
  router.push(key);
}
</script>

<template>
  <n-menu
    :value="activeKey"
    :options="menuOptions"
    @update:value="handleUpdate"
  />
</template>
