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

- Node shell / ephemeral debug containers, cordon/drain/taint.
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
