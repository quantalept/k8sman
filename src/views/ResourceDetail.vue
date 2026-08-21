<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useRoute } from "vue-router";
import { NCard, NSpace, NTabs, NTabPane, NText } from "naive-ui";
import ResourceYamlEditor from "../components/ResourceYamlEditor.vue";
import ResourceEventsTable from "../components/ResourceEventsTable.vue";
import DeleteResourceButton from "../components/DeleteResourceButton.vue";
import ScaleControl from "../components/ScaleControl.vue";
import RestartButton from "../components/RestartButton.vue";
import PinButton from "../components/PinButton.vue";
import RbacPermissionsTable from "../components/RbacPermissionsTable.vue";
import { getResource } from "../api/resources";
import { useClusterStore } from "../stores/cluster";
import { usePinnedStore } from "../stores/pinned";
import { findResourceKindByKind } from "../resourceKinds";

const route = useRoute();
const cluster = useClusterStore();
const pinned = usePinnedStore();

const kind = computed(() => String(route.params.kind));
const name = computed(() => String(route.params.name));
const namespace = computed(() => {
  const ns = route.query.ns;
  return typeof ns === "string" ? ns : undefined;
});

const config = computed(() => findResourceKindByKind(kind.value));
const currentReplicas = ref(0);

async function loadReplicas() {
  if (!config.value?.scalable) return;
  const contextName = cluster.currentContext;
  if (!contextName) return;
  try {
    const obj = await getResource(contextName, kind.value, namespace.value, name.value);
    currentReplicas.value = obj.spec?.replicas ?? 0;
  } catch {
    // Header controls just won't show a meaningful count; the YAML tab surfaces real errors.
  }
}

onMounted(() => {
  loadReplicas();
  pinned.recordVisit({ kind: kind.value, namespace: namespace.value, name: name.value });
});
watch([kind, name, namespace, () => cluster.currentContext], loadReplicas);
watch([kind, name, namespace], () =>
  pinned.recordVisit({ kind: kind.value, namespace: namespace.value, name: name.value }),
);
</script>

<template>
  <n-card :title="`${kind}: ${name}`">
    <template #header-extra>
      <n-space align="center">
        <ScaleControl
          v-if="config?.scalable"
          :kind="kind"
          :namespace="namespace"
          :name="name"
          :current-replicas="currentReplicas"
          @scaled="loadReplicas"
        />
        <RestartButton v-if="config?.restartable" :kind="kind" :namespace="namespace" :name="name" />
        <PinButton :kind="kind" :namespace="namespace" :name="name" />
        <DeleteResourceButton :kind="kind" :namespace="namespace" :name="name" />
      </n-space>
    </template>
    <n-space vertical size="large">
      <n-text v-if="namespace" depth="3">Namespace: {{ namespace }}</n-text>
      <n-tabs type="line">
        <n-tab-pane name="yaml" tab="YAML">
          <ResourceYamlEditor :kind="kind" :namespace="namespace" :name="name" />
        </n-tab-pane>
        <n-tab-pane name="events" tab="Events">
          <ResourceEventsTable :namespace="namespace" :involved-object-name="name" />
        </n-tab-pane>
        <n-tab-pane v-if="kind === 'ServiceAccount'" name="permissions" tab="Permissions">
          <RbacPermissionsTable :namespace="namespace" :name="name" />
        </n-tab-pane>
      </n-tabs>
    </n-space>
  </n-card>
</template>
