<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { useRouter } from "vue-router";
import { NModal, NCard, NInput, NEmpty, NTag, NText, NSpin } from "naive-ui";
import { useCommandPalette } from "../composables/useCommandPalette";
import { useClusterStore } from "../stores/cluster";
import { useSavedViewsStore } from "../stores/savedViews";
import { resourceKinds } from "../resourceKinds";
import { listResources } from "../api/resources";

interface PaletteItem {
  id: string;
  label: string;
  sublabel?: string;
  section: "Navigation" | "Saved Views" | "Resources";
  action: () => void;
}

const { isOpen, close } = useCommandPalette();
const router = useRouter();
const cluster = useClusterStore();
const savedViews = useSavedViewsStore();

const query = ref("");
const inputRef = ref<InstanceType<typeof NInput> | null>(null);
const selectedIndex = ref(0);
const searching = ref(false);
const resourceResults = ref<PaletteItem[]>([]);

const NAV_DESTINATIONS = [
  { label: "Dashboard", path: "/dashboard" },
  { label: "Events", path: "/events" },
  { label: "Pods", path: "/pods" },
  ...resourceKinds.map((k) => ({ label: k.label, path: `/${k.route}` })),
  { label: "Port Forwards", path: "/portforwards" },
  { label: "Settings", path: "/settings" },
];

function goTo(path: string) {
  router.push(path);
  close();
}

const navItems = computed<PaletteItem[]>(() => {
  const q = query.value.trim().toLowerCase();
  const matches = q
    ? NAV_DESTINATIONS.filter((d) => d.label.toLowerCase().includes(q))
    : NAV_DESTINATIONS;
  return matches.map((d) => ({
    id: `nav-${d.path}`,
    label: d.label,
    section: "Navigation",
    action: () => goTo(d.path),
  }));
});

const savedViewItems = computed<PaletteItem[]>(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return [];
  return savedViews.views
    .filter((v) => v.name.toLowerCase().includes(q))
    .map((v) => ({
      id: `view-${v.id}`,
      label: v.name,
      sublabel: `${v.kind}${v.namespace ? ` in ${v.namespace}` : ""}`,
      section: "Saved Views" as const,
      action: () => {
        const params = new URLSearchParams();
        if (v.namespace) params.set("ns", v.namespace);
        if (v.labelSelector) params.set("labels", v.labelSelector);
        const qs = params.toString();
        goTo(`/${v.route}${qs ? `?${qs}` : ""}`);
      },
    }));
});

function resourcePath(kind: string, obj: any): string {
  const namespace = obj.metadata?.namespace;
  if (kind === "Pod") return `/pods/${namespace}/${obj.metadata?.name}`;
  const query = namespace ? `?ns=${encodeURIComponent(namespace)}` : "";
  return `/resources/${kind}/${obj.metadata?.name}${query}`;
}

let searchGeneration = 0;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(query, (q) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  const term = q.trim();
  if (term.length < 2 || !cluster.currentContext) {
    resourceResults.value = [];
    searching.value = false;
    return;
  }
  debounceTimer = setTimeout(() => runResourceSearch(term), 300);
});

async function runResourceSearch(term: string) {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  const myGeneration = ++searchGeneration;
  searching.value = true;
  const kinds = ["Pod", ...resourceKinds.map((k) => k.kind)];
  const lowerTerm = term.toLowerCase();

  try {
    const results = await Promise.all(
      kinds.map(async (kind) => {
        try {
          const items = await listResources(contextName, kind);
          return items
            .filter((obj) => obj.metadata?.name?.toLowerCase().includes(lowerTerm))
            .slice(0, 5)
            .map(
              (obj): PaletteItem => ({
                id: `res-${kind}-${obj.metadata?.namespace}-${obj.metadata?.name}`,
                label: obj.metadata?.name,
                sublabel: `${kind}${obj.metadata?.namespace ? ` in ${obj.metadata.namespace}` : ""}`,
                section: "Resources",
                action: () => goTo(resourcePath(kind, obj)),
              }),
            );
        } catch {
          return [];
        }
      }),
    );
    if (myGeneration !== searchGeneration) return;
    resourceResults.value = results.flat().slice(0, 20);
  } finally {
    if (myGeneration === searchGeneration) searching.value = false;
  }
}

const allItems = computed(() => [...navItems.value, ...savedViewItems.value, ...resourceResults.value]);

watch(allItems, () => {
  if (selectedIndex.value >= allItems.value.length) selectedIndex.value = 0;
});

watch(isOpen, (open) => {
  if (open) {
    query.value = "";
    selectedIndex.value = 0;
    resourceResults.value = [];
    nextTick(() => inputRef.value?.focus());
  }
});

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowDown") {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, allItems.value.length - 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
  } else if (e.key === "Enter") {
    e.preventDefault();
    allItems.value[selectedIndex.value]?.action();
  } else if (e.key === "Escape") {
    close();
  }
}

function itemFlatIndex(item: PaletteItem): number {
  return allItems.value.findIndex((i) => i.id === item.id);
}
</script>

<template>
  <n-modal :show="isOpen" @update:show="(v) => !v && close()">
    <n-card style="width: 560px" content-style="padding: 0" :bordered="true">
      <n-input
        ref="inputRef"
        v-model:value="query"
        placeholder="Search resources or jump to a page…"
        size="large"
        style="border: none"
        @keydown="handleKeydown"
      />
      <div class="palette-results">
        <template v-for="section in (['Navigation', 'Saved Views', 'Resources'] as const)" :key="section">
          <template v-if="allItems.some((i) => i.section === section)">
            <div class="palette-section-label">{{ section }}</div>
            <div
              v-for="item in allItems.filter((i) => i.section === section)"
              :key="item.id"
              class="palette-item"
              :class="{ active: itemFlatIndex(item) === selectedIndex }"
              @click="item.action()"
              @mouseenter="selectedIndex = itemFlatIndex(item)"
            >
              <n-text>{{ item.label }}</n-text>
              <n-tag v-if="item.sublabel" size="tiny" :bordered="false">{{ item.sublabel }}</n-tag>
            </div>
          </template>
        </template>
        <n-spin v-if="searching" size="small" style="margin: 12px" />
        <n-empty v-if="allItems.length === 0 && !searching" description="No matches" style="padding: 24px" />
      </div>
    </n-card>
  </n-modal>
</template>

<style scoped>
.palette-results {
  max-height: 360px;
  overflow-y: auto;
  padding: 4px 0 8px;
}

.palette-section-label {
  padding: 6px 16px 2px;
  font-size: 11px;
  text-transform: uppercase;
  opacity: 0.5;
}

.palette-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
}

.palette-item.active {
  background: rgba(255, 255, 255, 0.08);
}
</style>
