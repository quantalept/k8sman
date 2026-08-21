import { createRouter, createWebHistory } from "vue-router";

const routes = [
  { path: "/", redirect: "/dashboard" },
  { path: "/dashboard", name: "dashboard", component: () => import("../views/Dashboard.vue") },
  { path: "/events", name: "events", component: () => import("../views/Events.vue") },
  { path: "/pods", name: "pods", component: () => import("../views/Pods.vue") },
  { path: "/pods/:namespace/:name", name: "pod-detail", component: () => import("../views/PodDetail.vue") },
  {
    path: "/:kindRoute",
    name: "resource-kind-list",
    component: () => import("../views/ResourceKindList.vue"),
  },
  {
    path: "/resources/:kind/:name",
    name: "resource-detail",
    component: () => import("../views/ResourceDetail.vue"),
  },
  {
    path: "/custom/:group/:version/:kind",
    name: "custom-resource-list",
    component: () => import("../views/CustomResourceList.vue"),
  },
  { path: "/helm", name: "helm-releases", component: () => import("../views/HelmReleases.vue") },
  {
    path: "/helm/:namespace/:name",
    name: "helm-release-detail",
    component: () => import("../views/HelmReleaseDetail.vue"),
  },
  { path: "/portforwards", name: "portforwards", component: () => import("../views/PortForwards.vue") },
  { path: "/settings", name: "settings", component: () => import("../views/Settings.vue") },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
