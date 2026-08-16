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

- Concurrent multi-cluster connections (tabs) instead of switch-and-disconnect
  — the client cache in `AppState` already supports holding multiple; this is
  mostly a frontend/UX change.
- Command palette (Cmd+K) + global resource search.
- Label/field selectors, saved views, pinned/recent resources.

## Tier 3 — Ecosystem integrations

- CRD-aware generic UI (discovery already returns them; needs a "Custom
  Resources" nav section + CRD printer-column support).
- Helm releases (list/values/history/rollback).
- RBAC viewer ("what can this ServiceAccount do").
- Prometheus-backed historical metrics — slots into the `MetricsProvider`
  trait already anticipated in `metrics.rs`.

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

Not started. Tiers are taken up one at a time; this file is updated with
**Done.** markers per item as they land, matching the convention in
`docs/architecture-plan.md`.
