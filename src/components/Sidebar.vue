<script setup lang="ts">
import { h, computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NIcon, NMenu, type MenuOption } from "naive-ui";
import {
  GridOutline,
  NotificationsOutline,
  CubeOutline,
  SwapHorizontalOutline,
  SettingsOutline,
  ExtensionPuzzleOutline,
  BoatOutline,
} from "@vicons/ionicons5";
import { resourceKinds, type ResourceKindConfig } from "../resourceKinds";
import { useCustomResourcesStore } from "../stores/customResources";
import { useClusterStore } from "../stores/cluster";

const route = useRoute();
const router = useRouter();
const customResources = useCustomResourcesStore();
const cluster = useClusterStore();

onMounted(() => customResources.load());
watch(() => cluster.currentContext, () => customResources.load());

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) });
}

const GROUP_ORDER: ResourceKindConfig["group"][] = [
  "Cluster",
  "Workloads",
  "Network",
  "Config",
  "Storage",
  "Access Control",
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

const customResourceMenu = computed<MenuOption | null>(() => {
  if (customResources.groups.length === 0) return null;
  return {
    type: "group",
    label: "Custom Resources",
    key: "group-custom-resources",
    children: customResources.groups.map((g) => ({
      type: "group",
      label: g.group,
      key: `custom-group-${g.group}`,
      children: g.kinds.map((k) => ({
        label: k.kind,
        key: `/custom/${k.group}/${k.version}/${k.kind}`,
        icon: renderIcon(ExtensionPuzzleOutline),
      })),
    })),
  };
});

const menuOptions = computed<MenuOption[]>(() => [
  { label: "Dashboard", key: "/dashboard", icon: renderIcon(GridOutline) },
  { label: "Events", key: "/events", icon: renderIcon(NotificationsOutline) },
  ...GROUP_ORDER.map(groupMenu).filter((g): g is MenuOption => g !== null),
  ...(customResourceMenu.value ? [customResourceMenu.value] : []),
  { label: "Helm Releases", key: "/helm", icon: renderIcon(BoatOutline) },
  { label: "Port Forwards", key: "/portforwards", icon: renderIcon(SwapHorizontalOutline) },
  { label: "Settings", key: "/settings", icon: renderIcon(SettingsOutline) },
]);

const activeKey = computed(() => {
  // Custom-resource menu keys are the full path (they're not single-segment routes).
  if (route.path.startsWith("/custom/")) return route.path;
  return "/" + route.path.split("/")[1];
});

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
