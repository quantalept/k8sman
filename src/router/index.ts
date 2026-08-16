import { createRouter, createWebHistory } from "vue-router";

const routes = [
  { path: "/", redirect: "/dashboard" },
  { path: "/dashboard", name: "dashboard", component: () => import("../views/Dashboard.vue") },
  { path: "/namespaces", name: "namespaces", component: () => import("../views/Namespaces.vue") },
  { path: "/nodes", name: "nodes", component: () => import("../views/Nodes.vue") },
  { path: "/pods", name: "pods", component: () => import("../views/Pods.vue") },
  { path: "/pods/:namespace/:name", name: "pod-detail", component: () => import("../views/PodDetail.vue") },
  { path: "/services", name: "services", component: () => import("../views/Services.vue") },
  { path: "/configmaps", name: "configmaps", component: () => import("../views/ConfigMaps.vue") },
  { path: "/secrets", name: "secrets", component: () => import("../views/Secrets.vue") },
  { path: "/storage", name: "storage", component: () => import("../views/Storage.vue") },
  { path: "/portforwards", name: "portforwards", component: () => import("../views/PortForwards.vue") },
  { path: "/settings", name: "settings", component: () => import("../views/Settings.vue") },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
