<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute } from "vue-router";
import { NCard, NSpace, NTabs, NTabPane, NText, NDescriptions, NDescriptionsItem, NTag } from "naive-ui";
import ResourceYamlEditor from "../components/ResourceYamlEditor.vue";
import ResourceEventsTable from "../components/ResourceEventsTable.vue";
import ResourceTable from "../components/ResourceTable.vue";
import DeleteResourceButton from "../components/DeleteResourceButton.vue";
import PinButton from "../components/PinButton.vue";
import CordonButton from "../components/CordonButton.vue";
import DrainNodeButton from "../components/DrainNodeButton.vue";
import NodeShellButton from "../components/NodeShellButton.vue";
import TaintEditor from "../components/TaintEditor.vue";
import { getResource } from "../api/resources";
import { useClusterStore } from "../stores/cluster";
import { usePinnedStore } from "../stores/pinned";

const route = useRoute();
const cluster = useClusterStore();
const pinned = usePinnedStore();

const name = computed(() => String(route.params.name));
const node = ref<any>(null);

async function load() {
  const contextName = cluster.currentContext;
  if (!contextName) return;
  try {
    node.value = await getResource(contextName, "Node", undefined, name.value);
  } catch {
    // The YAML/Events tabs surface real errors; header just won't have data to show.
  }
}

onMounted(() => {
  load();
  pinned.recordVisit({ kind: "Node", namespace: undefined, name: name.value });
});
watch([name, () => cluster.currentContext], load);
watch(name, () => pinned.recordVisit({ kind: "Node", namespace: undefined, name: name.value }));

const unschedulable = computed(() => Boolean(node.value?.spec?.unschedulable));
const taints = computed(() => node.value?.spec?.taints ?? []);
const conditions = computed(() => node.value?.status?.conditions ?? []);
const capacity = computed(() => node.value?.status?.capacity ?? {});
const allocatable = computed(() => node.value?.status?.allocatable ?? {});
</script>

<template>
  <n-card :title="`Node: ${name}`">
    <template #header-extra>
      <n-space align="center">
        <CordonButton :name="name" :unschedulable="unschedulable" @changed="load" />
        <DrainNodeButton :name="name" />
        <NodeShellButton :name="name" />
        <PinButton kind="Node" :name="name" />
        <DeleteResourceButton kind="Node" :name="name" back-path="/nodes" />
      </n-space>
    </template>
    <n-space vertical size="large">
      <n-tabs type="line">
        <n-tab-pane name="overview" tab="Overview">
          <n-space vertical size="large">
            <n-descriptions label-placement="left" :column="1" bordered size="small">
              <n-descriptions-item label="Schedulable">
                <n-tag :type="unschedulable ? 'error' : 'success'" size="small" round>
                  {{ unschedulable ? "Cordoned" : "Schedulable" }}
                </n-tag>
              </n-descriptions-item>
              <n-descriptions-item label="Conditions">
                <n-space :size="4">
                  <n-tag
                    v-for="c in conditions"
                    :key="c.type"
                    size="small"
                    round
                    :type="c.status === 'True' ? (c.type === 'Ready' ? 'success' : 'warning') : 'default'"
                  >
                    {{ c.type }}
                  </n-tag>
                </n-space>
              </n-descriptions-item>
              <n-descriptions-item label="Capacity">
                CPU {{ capacity.cpu ?? "-" }} / Memory {{ capacity.memory ?? "-" }} / Pods {{ capacity.pods ?? "-" }}
              </n-descriptions-item>
              <n-descriptions-item label="Allocatable">
                CPU {{ allocatable.cpu ?? "-" }} / Memory {{ allocatable.memory ?? "-" }} / Pods
                {{ allocatable.pods ?? "-" }}
              </n-descriptions-item>
            </n-descriptions>
            <div>
              <n-text depth="3" style="display: block; margin-bottom: 8px">Taints</n-text>
              <TaintEditor :name="name" :taints="taints" @changed="load" />
            </div>
          </n-space>
        </n-tab-pane>
        <n-tab-pane name="pods" tab="Pods on node">
          <ResourceTable
            kind="Pod"
            :field-selector="`spec.nodeName=${name}`"
            :show-namespace="true"
          />
        </n-tab-pane>
        <n-tab-pane name="yaml" tab="YAML">
          <ResourceYamlEditor kind="Node" :name="name" />
        </n-tab-pane>
        <n-tab-pane name="events" tab="Events">
          <ResourceEventsTable :involved-object-name="name" />
        </n-tab-pane>
      </n-tabs>
    </n-space>
  </n-card>
</template>
