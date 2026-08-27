<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { VueFlow, type Node as FlowNode, type Edge as FlowEdge, type NodeMouseEvent } from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import { Controls } from "@vue-flow/controls";
import dagre from "dagre";
import { NAlert, NSpin, NText } from "naive-ui";
import TopologyGraphNode from "./TopologyGraphNode.vue";
import { useResourceTopology, type TopologyGraphNode as TopoNode } from "../composables/useResourceTopology";
import type { ResourceRef } from "../topology/relationships";
import "@vue-flow/core/dist/style.css";
import "@vue-flow/core/dist/theme-default.css";
import "@vue-flow/controls/dist/style.css";

const props = defineProps<{
  kind: string;
  namespace?: string;
  name: string;
}>();

const focal = ref<ResourceRef>({ kind: props.kind, namespace: props.namespace, name: props.name });
watch(
  () => [props.kind, props.namespace, props.name],
  () => {
    focal.value = { kind: props.kind, namespace: props.namespace, name: props.name };
  },
);

const { nodes, edges, loading, error, recenter } = useResourceTopology(focal);

const NODE_WIDTH = 180;
const NODE_HEIGHT = 56;

const layout = computed(() => {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: "TB", nodesep: 32, ranksep: 64 });

  for (const n of nodes.value) g.setNode(n.id, { width: NODE_WIDTH, height: NODE_HEIGHT });
  for (const e of edges.value) g.setEdge(e.source, e.target);
  dagre.layout(g);

  const flowNodes: FlowNode[] = nodes.value.map((n) => {
    const pos = g.node(n.id);
    return {
      id: n.id,
      type: "resource",
      position: { x: pos.x - NODE_WIDTH / 2, y: pos.y - NODE_HEIGHT / 2 },
      data: n,
    };
  });

  const flowEdges: FlowEdge[] = edges.value.map((e) => ({
    id: e.id,
    source: e.source,
    target: e.target,
    label: e.label,
    animated: false,
  }));

  return { flowNodes, flowEdges };
});

function onNodeClick(event: NodeMouseEvent) {
  const data = event.node.data as TopoNode;
  if (data.focal) return;
  recenter(data);
}
</script>

<template>
  <n-alert v-if="error" type="error" :title="error" closable style="margin-bottom: 12px" />
  <n-spin :show="loading">
    <div class="topology-canvas">
      <n-text v-if="!loading && nodes.length <= 1" depth="3">No relationships found for this resource.</n-text>
      <VueFlow
        v-else
        :nodes="layout.flowNodes"
        :edges="layout.flowEdges"
        :nodes-draggable="true"
        :nodes-connectable="false"
        :edges-updatable="false"
        fit-view-on-init
        @node-click="onNodeClick"
      >
        <template #node-resource="nodeProps">
          <TopologyGraphNode :data="nodeProps.data" />
        </template>
        <Background />
        <Controls :show-interactive="false" />
      </VueFlow>
    </div>
  </n-spin>
</template>

<style scoped>
.topology-canvas {
  height: 500px;
  border-radius: 6px;
  overflow: hidden;
  border: 1px solid var(--n-border-color, #333);
}
</style>
