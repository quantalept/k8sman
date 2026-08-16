<script setup lang="ts">
import { h, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NIcon, NMenu, type MenuOption } from "naive-ui";
import {
  GridOutline,
  NotificationsOutline,
  CubeOutline,
  SwapHorizontalOutline,
  SettingsOutline,
} from "@vicons/ionicons5";
import { resourceKinds, type ResourceKindConfig } from "../resourceKinds";

const route = useRoute();
const router = useRouter();

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) });
}

const GROUP_ORDER: ResourceKindConfig["group"][] = [
  "Cluster",
  "Workloads",
  "Network",
  "Config",
  "Storage",
];

function groupMenu(group: ResourceKindConfig["group"]): MenuOption | null {
  const children: MenuOption[] = resourceKinds
    .filter((r) => r.group === group)
    .map((r) => ({ label: r.label, key: `/${r.route}`, icon: renderIcon(r.icon) }));

  // Pods aren't in the registry (they have a bespoke view with metrics/exec/logs/etc.),
  // but they're still a workload kind and belong in the same nav group as the rest.
  if (group === "Workloads") {
    children.unshift({ label: "Pods", key: "/pods", icon: renderIcon(CubeOutline) });
  }

  if (children.length === 0) return null;
  return { type: "group", label: group, key: `group-${group}`, children };
}

const menuOptions: MenuOption[] = [
  { label: "Dashboard", key: "/dashboard", icon: renderIcon(GridOutline) },
  { label: "Events", key: "/events", icon: renderIcon(NotificationsOutline) },
  ...GROUP_ORDER.map(groupMenu).filter((g): g is MenuOption => g !== null),
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
