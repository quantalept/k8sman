import { h, type Component } from "vue";
import { NTag, type DataTableColumns } from "naive-ui";
import {
  FolderOutline,
  ServerOutline,
  GitNetworkOutline,
  DocumentTextOutline,
  LockClosedOutline,
  LayersOutline,
  CopyOutline,
  AlbumsOutline,
  AppsOutline,
  FlashOutline,
  TimeOutline,
  GlobeOutline,
  SaveOutline,
  BookmarkOutline,
  TrendingUpOutline,
  PersonOutline,
  KeyOutline,
  ShieldCheckmarkOutline,
  LinkOutline,
  GitCompareOutline,
} from "@vicons/ionicons5";

export interface ResourceKindConfig {
  /** The Kubernetes Kind, used to resolve the resource via discovery. */
  kind: string;
  /** Sidebar route segment, e.g. "deployments" -> /deployments. */
  route: string;
  label: string;
  namespaced: boolean;
  icon: Component;
  /** Sidebar section grouping. */
  group: "Cluster" | "Workloads" | "Network" | "Config" | "Storage" | "Access Control";
  /** Whether ResourceDetail should show a replica-count Scale control. */
  scalable?: boolean;
  /** Whether ResourceDetail should show a "Restart" (rollout restart) button. */
  restartable?: boolean;
  /** Extra columns beyond ResourceTable's built-in Name/Namespace/Age. */
  columns?: DataTableColumns<any>;
}

// The stable set of built-in Kubernetes API groups. Anything discovery returns outside this
// set is treated as a CRD (verified against a real cluster: correctly separates cert-manager,
// calico, istio, nvidia, and node-feature-discovery groups from the built-ins).
export const KNOWN_BUILTIN_GROUPS = new Set([
  "",
  "apps",
  "batch",
  "networking.k8s.io",
  "rbac.authorization.k8s.io",
  "autoscaling",
  "policy",
  "storage.k8s.io",
  "apiextensions.k8s.io",
  "metrics.k8s.io",
  "admissionregistration.k8s.io",
  "apiregistration.k8s.io",
  "authentication.k8s.io",
  "authorization.k8s.io",
  "certificates.k8s.io",
  "coordination.k8s.io",
  "discovery.k8s.io",
  "events.k8s.io",
  "flowcontrol.apiserver.k8s.io",
  "node.k8s.io",
  "scheduling.k8s.io",
]);

function statusTag(text: string, type: "success" | "warning" | "error" | "default") {
  return h(NTag, { type, size: "small", round: true }, { default: () => text });
}

export const resourceKinds: ResourceKindConfig[] = [
  {
    kind: "Namespace",
    route: "namespaces",
    label: "Namespaces",
    namespaced: false,
    icon: FolderOutline,
    group: "Cluster",
    columns: [{ title: "Status", key: "status", render: (row: any) => row.status?.phase ?? "-" }],
  },
  {
    kind: "Node",
    route: "nodes",
    label: "Nodes",
    namespaced: false,
    icon: ServerOutline,
    group: "Cluster",
    columns: [
      {
        title: "Status",
        key: "status",
        render: (row: any) => {
          const ready = (row.status?.conditions ?? []).some(
            (c: any) => c.type === "Ready" && c.status === "True",
          );
          return statusTag(ready ? "Ready" : "Not Ready", ready ? "success" : "error");
        },
      },
      {
        title: "Version",
        key: "version",
        render: (row: any) => row.status?.nodeInfo?.kubeletVersion ?? "-",
      },
    ],
  },
  {
    kind: "Deployment",
    route: "deployments",
    label: "Deployments",
    namespaced: true,
    icon: LayersOutline,
    group: "Workloads",
    scalable: true,
    restartable: true,
    columns: [
      {
        title: "Ready",
        key: "ready",
        render: (row: any) => `${row.status?.readyReplicas ?? 0}/${row.spec?.replicas ?? 0}`,
      },
      { title: "Up-to-date", key: "updated", render: (row: any) => row.status?.updatedReplicas ?? 0 },
      { title: "Available", key: "available", render: (row: any) => row.status?.availableReplicas ?? 0 },
    ],
  },
  {
    kind: "ReplicaSet",
    route: "replicasets",
    label: "ReplicaSets",
    namespaced: true,
    icon: CopyOutline,
    group: "Workloads",
    scalable: true,
    columns: [
      { title: "Desired", key: "desired", render: (row: any) => row.spec?.replicas ?? 0 },
      { title: "Current", key: "current", render: (row: any) => row.status?.replicas ?? 0 },
      { title: "Ready", key: "ready", render: (row: any) => row.status?.readyReplicas ?? 0 },
    ],
  },
  {
    kind: "StatefulSet",
    route: "statefulsets",
    label: "StatefulSets",
    namespaced: true,
    icon: AlbumsOutline,
    group: "Workloads",
    scalable: true,
    restartable: true,
    columns: [
      {
        title: "Ready",
        key: "ready",
        render: (row: any) => `${row.status?.readyReplicas ?? 0}/${row.spec?.replicas ?? 0}`,
      },
    ],
  },
  {
    kind: "DaemonSet",
    route: "daemonsets",
    label: "DaemonSets",
    namespaced: true,
    icon: AppsOutline,
    group: "Workloads",
    restartable: true,
    columns: [
      { title: "Desired", key: "desired", render: (row: any) => row.status?.desiredNumberScheduled ?? 0 },
      { title: "Current", key: "current", render: (row: any) => row.status?.currentNumberScheduled ?? 0 },
      { title: "Ready", key: "ready", render: (row: any) => row.status?.numberReady ?? 0 },
    ],
  },
  {
    kind: "Job",
    route: "jobs",
    label: "Jobs",
    namespaced: true,
    icon: FlashOutline,
    group: "Workloads",
    columns: [
      {
        title: "Completions",
        key: "completions",
        render: (row: any) => `${row.status?.succeeded ?? 0}/${row.spec?.completions ?? 1}`,
      },
    ],
  },
  {
    kind: "CronJob",
    route: "cronjobs",
    label: "CronJobs",
    namespaced: true,
    icon: TimeOutline,
    group: "Workloads",
    columns: [
      { title: "Schedule", key: "schedule", render: (row: any) => row.spec?.schedule ?? "-" },
      {
        title: "Suspended",
        key: "suspend",
        render: (row: any) => (row.spec?.suspend ? "Yes" : "No"),
      },
      {
        title: "Last Schedule",
        key: "lastSchedule",
        render: (row: any) => row.status?.lastScheduleTime ?? "never",
      },
    ],
  },
  {
    kind: "Service",
    route: "services",
    label: "Services",
    namespaced: true,
    icon: GitNetworkOutline,
    group: "Network",
    columns: [
      { title: "Type", key: "type", render: (row: any) => row.spec?.type ?? "ClusterIP" },
      { title: "Cluster IP", key: "clusterIP", render: (row: any) => row.spec?.clusterIP ?? "-" },
      {
        title: "Ports",
        key: "ports",
        render: (row: any) =>
          (row.spec?.ports ?? [])
            .map((p: any) => `${p.port}${p.protocol ? "/" + p.protocol : ""}`)
            .join(", ") || "-",
      },
    ],
  },
  {
    kind: "Ingress",
    route: "ingresses",
    label: "Ingresses",
    namespaced: true,
    icon: GlobeOutline,
    group: "Network",
    columns: [
      { title: "Class", key: "class", render: (row: any) => row.spec?.ingressClassName ?? "-" },
      {
        title: "Hosts",
        key: "hosts",
        render: (row: any) =>
          (row.spec?.rules ?? []).map((r: any) => r.host).filter(Boolean).join(", ") || "*",
      },
    ],
  },
  {
    kind: "ConfigMap",
    route: "configmaps",
    label: "ConfigMaps",
    namespaced: true,
    icon: DocumentTextOutline,
    group: "Config",
    columns: [
      { title: "Keys", key: "keys", render: (row: any) => Object.keys(row.data ?? {}).length },
    ],
  },
  {
    kind: "Secret",
    route: "secrets",
    label: "Secrets",
    namespaced: true,
    icon: LockClosedOutline,
    group: "Config",
    columns: [
      { title: "Type", key: "type", render: (row: any) => row.type ?? "Opaque" },
      { title: "Keys", key: "keys", render: (row: any) => Object.keys(row.data ?? {}).length },
    ],
  },
  {
    kind: "HorizontalPodAutoscaler",
    route: "hpas",
    label: "HPAs",
    namespaced: true,
    icon: TrendingUpOutline,
    group: "Config",
    columns: [
      {
        title: "Reference",
        key: "reference",
        render: (row: any) => row.spec?.scaleTargetRef?.name ?? "-",
      },
      {
        title: "Min/Max",
        key: "minmax",
        render: (row: any) => `${row.spec?.minReplicas ?? "-"}/${row.spec?.maxReplicas ?? "-"}`,
      },
      { title: "Replicas", key: "replicas", render: (row: any) => row.status?.currentReplicas ?? 0 },
    ],
  },
  {
    kind: "PersistentVolume",
    route: "persistentvolumes",
    label: "Persistent Volumes",
    namespaced: false,
    icon: SaveOutline,
    group: "Storage",
    columns: [
      { title: "Capacity", key: "capacity", render: (row: any) => row.spec?.capacity?.storage ?? "-" },
      { title: "Status", key: "status", render: (row: any) => row.status?.phase ?? "-" },
      { title: "Claim", key: "claim", render: (row: any) => row.spec?.claimRef?.name ?? "-" },
    ],
  },
  {
    kind: "PersistentVolumeClaim",
    route: "persistentvolumeclaims",
    label: "Persistent Volume Claims",
    namespaced: true,
    icon: BookmarkOutline,
    group: "Storage",
    columns: [
      { title: "Status", key: "status", render: (row: any) => row.status?.phase ?? "-" },
      { title: "Volume", key: "volume", render: (row: any) => row.spec?.volumeName ?? "-" },
      {
        title: "Capacity",
        key: "capacity",
        render: (row: any) => row.status?.capacity?.storage ?? "-",
      },
    ],
  },
  {
    kind: "ServiceAccount",
    route: "serviceaccounts",
    label: "Service Accounts",
    namespaced: true,
    icon: PersonOutline,
    group: "Access Control",
  },
  {
    kind: "Role",
    route: "roles",
    label: "Roles",
    namespaced: true,
    icon: KeyOutline,
    group: "Access Control",
    columns: [
      { title: "Rules", key: "rules", render: (row: any) => (row.rules ?? []).length },
    ],
  },
  {
    kind: "ClusterRole",
    route: "clusterroles",
    label: "Cluster Roles",
    namespaced: false,
    icon: ShieldCheckmarkOutline,
    group: "Access Control",
    columns: [
      { title: "Rules", key: "rules", render: (row: any) => (row.rules ?? []).length },
    ],
  },
  {
    kind: "RoleBinding",
    route: "rolebindings",
    label: "Role Bindings",
    namespaced: true,
    icon: LinkOutline,
    group: "Access Control",
    columns: [
      {
        title: "Role",
        key: "role",
        render: (row: any) => `${row.roleRef?.kind ?? "-"}/${row.roleRef?.name ?? "-"}`,
      },
    ],
  },
  {
    kind: "ClusterRoleBinding",
    route: "clusterrolebindings",
    label: "Cluster Role Bindings",
    namespaced: false,
    icon: GitCompareOutline,
    group: "Access Control",
    columns: [
      {
        title: "Role",
        key: "role",
        render: (row: any) => `${row.roleRef?.kind ?? "-"}/${row.roleRef?.name ?? "-"}`,
      },
    ],
  },
];

export function findResourceKind(route: string): ResourceKindConfig | undefined {
  return resourceKinds.find((r) => r.route === route);
}

export function findResourceKindByKind(kind: string): ResourceKindConfig | undefined {
  return resourceKinds.find((r) => r.kind === kind);
}
