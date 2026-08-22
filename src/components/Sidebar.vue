<script setup lang="ts">
import { h, computed, onMounted, watch, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NIcon, NMenu, NButton, NSpace, NTooltip, type MenuOption } from "naive-ui";
import {
  GridOutline,
  NotificationsOutline,
  CubeOutline,
  SwapHorizontalOutline,
  SettingsOutline,
  ExtensionPuzzleOutline,
  BoatOutline,
  ExpandOutline,
  ContractOutline,
} from "@vicons/ionicons5";
import { resourceKinds, type ResourceKindConfig } from "../resourceKinds";
import { useCustomResourcesStore } from "../stores/customResources";
import { useClusterStore } from "../stores/cluster";
import { getUiState, setUiState } from "../api/uiState";

const route = useRoute();
const router = useRouter();
const customResources = useCustomResourcesStore();
const cluster = useClusterStore();

const GROUP_ORDER: ResourceKindConfig["group"][] = [
  "Cluster",
  "Workloads",
  "Network",
  "Config",
  "Storage",
  "Access Control",
];

const KEY_EXPANDED = "sidebarExpandedKeys";
// Sections start expanded by default - matches the previous always-open behaviour until the
// user deliberately collapses something, which is then remembered.
const expandedKeys = ref<string[]>([
  ...GROUP_ORDER.map((g) => `group-${g}`),
  "group-custom-resources",
]);

onMounted(async () => {
  customResources.load();
  const saved = await getUiState<string[]>(KEY_EXPANDED);
  if (saved) expandedKeys.value = saved;
});
watch(() => cluster.currentContext, () => customResources.load());

function handleExpandedKeysChange(keys: string[]) {
  expandedKeys.value = keys;
  void setUiState(KEY_EXPANDED, keys);
}

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) });
}

// Deliberately not naive-ui's `type: "group"` menu items - those render as static, always-open
// dividers with no click handling. A plain item with `children` (and a stable `key`) becomes a
// real collapsible submenu instead, which is what makes sections minimizable.
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
  return { label: group, key: `group-${group}`, children };
}

const customResourceMenu = computed<MenuOption | null>(() => {
  if (customResources.groups.length === 0) return null;
  return {
    label: "Custom Resources",
    key: "group-custom-resources",
    children: customResources.groups.map((g) => ({
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

function collectExpandableKeys(options: MenuOption[]): string[] {
  const keys: string[] = [];
  for (const opt of options) {
    if (opt.children && opt.key) {
      keys.push(String(opt.key));
      keys.push(...collectExpandableKeys(opt.children));
    }
  }
  return keys;
}

function expandAll() {
  handleExpandedKeysChange(collectExpandableKeys(menuOptions.value));
}

function collapseAll() {
  handleExpandedKeysChange([]);
}
</script>

<template>
  <div class="sidebar">
    <n-space class="sidebar-toolbar" :size="4" justify="end">
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button size="tiny" quaternary circle @click="expandAll">
            <template #icon><n-icon><ExpandOutline /></n-icon></template>
          </n-button>
        </template>
        Expand all
      </n-tooltip>
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button size="tiny" quaternary circle @click="collapseAll">
            <template #icon><n-icon><ContractOutline /></n-icon></template>
          </n-button>
        </template>
        Collapse all
      </n-tooltip>
    </n-space>
    <n-menu
      :value="activeKey"
      :options="menuOptions"
      :expanded-keys="expandedKeys"
      @update:expanded-keys="handleExpandedKeysChange"
      @update:value="handleUpdate"
    />
  </div>
</template>

<style scoped>
.sidebar-toolbar {
  padding: 0 8px 4px;
}
</style>
