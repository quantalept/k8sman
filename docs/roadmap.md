# k8sman — v2 Roadmap

## Context

v1 (see `docs/architecture-plan.md`) covers cluster connectivity, generic resource
browsing/watching, log tailing, pod exec, port-forward, file copy, and basic
metrics — all built on a discovery-driven backend (works for CRDs, not just the
six built-in kinds it currently has views for) and a `MetricsProvider` trait
already shaped for a second backend. This document lists the functionality
beyond that, grouped into tiers by how directly each builds on what already
exists. Tiers are worked through one at a time, in order, unless reprioritized.

## Tier 1 — Core workflow completeness

The biggest functional gap vs. OpenLens/Freelens right now: read-only browsing
of six resource kinds, no editing, no events. **Done** — see `src/resourceKinds.ts`,
the registry that drives both list views and detail-page actions.

- Full resource kind coverage: Deployments, ReplicaSets, StatefulSets,
  DaemonSets, Jobs, CronJobs, Ingresses, PVs/PVCs, HPAs — using the existing
  generic `ResourceTable`/discovery machinery, so mostly new views + column
  configs, not new backend plumbing. **Done.**
- A real resource **detail page** per kind (not just Pods): YAML view,
  Events tab. **Done** (owner-chain navigation — e.g. Deployment → ReplicaSet
  → Pods — not included; left for a later pass).
- **Edit YAML** (Monaco editor) with apply, plus delete/scale/restart-rollout
  actions — all destructive, so confirmation dialogs are load-bearing here.
  **Done**, server-side apply via `kubectl`-equivalent field-manager patches;
  scale/restart implemented as merge-patches on Deployment/ReplicaSet/
  StatefulSet/DaemonSet (not the generic `/scale` subresource, so custom
  resources with their own scale shape aren't covered).
- **Events** view, cluster-wide and per-resource. **Done.**

## Tier 2 — Multi-cluster & productivity

**Done.**

- Concurrent multi-cluster connections (tabs) instead of switch-and-disconnect
  — the client cache in `AppState` already supports holding multiple; this is
  mostly a frontend/UX change. **Done** as a lightweight tab strip
  (`ClusterSwitcher.vue`/`stores/cluster.ts`): switching tabs changes which
  cluster the *current* page queries, no reconnect delay. Fully independent
  per-tab navigation state was scoped out as a much bigger build.
- Command palette (Cmd+K) + global resource search. **Done**
  (`CommandPalette.vue`) — navigation, saved views, and a debounced
  cross-kind resource search (name substring match, capped results).
- Label/field selectors, saved views, pinned/recent resources. **Done** —
  label selectors on `list_resources`/`start_watch` (mirrors the Tier 1
  `field_selector` addition); saved views and pinned/recent persisted via a
  frontend-owned `ui-state.json` store (`api/uiState.ts`), separate from the
  backend's own kubeconfig/context settings store.

## Tier 3 — Ecosystem integrations

**Done**, with two scope notes decided during implementation.

- CRD-aware generic UI. **Done** — `list_resource_kinds` (from Tier 1)
  already returns CRDs; a static `KNOWN_BUILTIN_GROUPS` set in
  `resourceKinds.ts` separates them from built-ins (verified against the
  real cluster's actual CRDs: cert-manager, calico, istio, nvidia,
  node-feature-discovery). Printer columns come from the CRD's own
  `additionalPrinterColumns`, evaluated with a small JSONPath subset
  (`jsonPath.ts`) that had to support K8s's `[?(@.field=="value")]` filter
  syntax, not just dot-paths — real cert-manager columns (Ready/Status) use
  exactly that, caught by testing against the live CRD rather than assuming.
  Correction (found while investigating a live report of missing Istio
  CRDs, fixed alongside Tier 4): the "verified" claim above missed two bugs
  that only show up on clusters shaped like a real Istio install.
  `list_resource_kinds` (`resources.rs`) used `ApiGroup::recommended_resources()`,
  which only returns kinds served at a group's single "recommended" version —
  Istio's `networking.istio.io` mixes kinds pinned to different versions
  (some at `v1`, others still only `v1beta1`/`v1alpha3`), so any kind not at
  that one version silently never reached the frontend; switched to
  `resources_by_stability()`, which picks each kind's own most-stable
  version instead of one version for the whole group. Separately, kube's
  `Discovery::run()` has no per-group error isolation — one broken group
  (dead conversion webhook, stale aggregated APIService, RBAC-forbidden
  group) fails discovery for the *entire* cluster, and the frontend
  (`customResources.ts`) caught that into an `error` field nothing ever
  read, so the whole "Custom Resources" sidebar section vanished silently
  with no indication why. Replaced the single atomic `Discovery::run()` call
  with a per-group walk (`ResilientDiscovery`/`run_discovery` in
  `resources.rs`, built on kube's public `discovery::group()` oneshot
  helper) that skips and logs a failing group instead of failing everything,
  and wired the frontend error into a visible sidebar alert (`Sidebar.vue`).
- Helm releases. **Done, read-only** (list/values/manifest/history) — user
  decision: no `helm` binary dependency, releases decoded directly from
  their `helm.sh/release.v1` Secrets (`helm.rs`), verified against real
  releases already on the cluster (cert-manager, learn2write, gpu-operator).
  Rollback deliberately not implemented (resource pruning/hooks are easy to
  get subtly wrong).
- RBAC viewer. **Done** — binding-aggregation approach (`rbac.rs`), not
  `SubjectAccessReview` (doesn't need extra permissions beyond read).
  ServiceAccount/Role/ClusterRole/RoleBinding/ClusterRoleBinding registered
  as normal resource kinds (free list/detail/YAML/delete); a "Permissions"
  tab on ServiceAccount detail pages resolves bound Role/ClusterRole rules.
- Prometheus-backed historical metrics. **Done for cluster-level Dashboard
  metrics**; per-pod Prometheus metrics not done (flagged as a nice-to-have
  in the plan, `usePodMetrics.ts` still metrics-server-only). Correction to
  this doc's earlier claim: there was no `MetricsProvider` trait already in
  `metrics.rs` - added parallel commands instead, frontend picks the source.
  Surfaced a real bug: `kube`'s and `reqwest`'s independent rustls stacks
  need an explicit `CryptoProvider::install_default()` at startup or every
  TLS call (including kube's own) panics - fixed in `lib.rs`.

## Tier 4 — Debugging & operations

- Node shell / ephemeral debug containers, cordon/drain/taint. **Done** — a dedicated
  `NodeDetail.vue` (Node now gets its own detail page, like Pod, instead of the generic
  `ResourceDetail.vue`) adds cordon/uncordon (`spec.unschedulable` patch), a taint editor
  (`nodes.rs`, read-modify-write on `spec.taints` since JSON merge-patch replaces arrays
  wholesale), and a node-shell button that creates a privileged `hostPID`/`hostNetwork`
  pod pinned to the node via `spec.nodeName` (bypasses the scheduler, so it works even
  cordoned) with `/` mounted at `/host`, reusing the existing `ExecTerminal`/`start_exec`
  plumbing unchanged. Drain (`start_node_drain`) is a single-pass eviction over
  `Api<Pod>::evict` (kube's eviction subresource) that skips DaemonSet-owned and
  static/mirror pods and streams per-pod progress the same way `start_watch` streams
  resource events — no PDB-aware retry loop like `kubectl drain`, so a pod blocked by a
  PodDisruptionBudget surfaces as an error rather than being retried. Also added an
  ephemeral-debug-container tab on `PodDetail.vue` (kube's `ephemeralcontainers`
  subresource, read-append-replace since it doesn't merge lists) using the same
  attach/exec path — no `exec.rs` changes needed since it already takes an arbitrary
  container name. No dependency bump: kube 0.99/k8s-openapi 0.24 already cover all of
  this.
- Crash-loop/restart alerts, desktop notifications.
- Resource topology graph.

## Tier 5 — Polish & shipping

- Light/dark theme toggle (app is hardcoded dark right now).
- Secret reveal/hide toggle, RBAC-aware UI (hide actions the user can't
  perform).
- Tauri auto-updater + release pipeline, multi-window support.

## Status

Tiers 1, 2, and 3 done. Tiers are taken up one at a time; this file is
updated with **Done.** markers per item as they land, matching the
convention in `docs/architecture-plan.md`.
