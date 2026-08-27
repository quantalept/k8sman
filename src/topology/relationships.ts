/** A reference to a resource, identifying enough to fetch or navigate to it. */
export interface ResourceRef {
  kind: string;
  namespace?: string;
  name: string;
}

export interface RelatedRef extends ResourceRef {
  /** Short description of the relationship, shown as the edge label. */
  label: string;
}

function dedupeRefs(refs: RelatedRef[]): RelatedRef[] {
  const seen = new Set<string>();
  const out: RelatedRef[] = [];
  for (const r of refs) {
    const key = `${r.kind}/${r.namespace ?? ""}/${r.name}`;
    if (seen.has(key)) continue;
    seen.add(key);
    out.push(r);
  }
  return out;
}

/**
 * Direct parents/dependencies of `obj`, derived purely from its own spec/metadata - no
 * fetching required. Covers the owner chain (`metadata.ownerReferences`) plus
 * spec-embedded references (Ingress->Service, HPA->scaleTargetRef, Pod->ConfigMap/
 * Secret/PVC/Node/ServiceAccount, PVC<->PV).
 */
export function getParentRefs(kind: string, obj: any): RelatedRef[] {
  const namespace = obj?.metadata?.namespace;
  const refs: RelatedRef[] = [];

  for (const owner of obj?.metadata?.ownerReferences ?? []) {
    refs.push({ kind: owner.kind, namespace, name: owner.name, label: "Owned by" });
  }

  if (kind === "Ingress") {
    const backends: any[] = [];
    if (obj?.spec?.defaultBackend?.service) backends.push(obj.spec.defaultBackend.service);
    for (const rule of obj?.spec?.rules ?? []) {
      for (const path of rule.http?.paths ?? []) {
        if (path.backend?.service) backends.push(path.backend.service);
      }
    }
    for (const svc of backends) {
      if (svc?.name) refs.push({ kind: "Service", namespace, name: svc.name, label: "Routes to" });
    }
  }

  if (kind === "HorizontalPodAutoscaler") {
    const target = obj?.spec?.scaleTargetRef;
    if (target?.name) {
      refs.push({ kind: target.kind, namespace, name: target.name, label: "Scales" });
    }
  }

  if (kind === "Pod") {
    const spec = obj?.spec ?? {};
    if (spec.nodeName) refs.push({ kind: "Node", name: spec.nodeName, label: "Runs on" });
    if (spec.serviceAccountName) {
      refs.push({ kind: "ServiceAccount", namespace, name: spec.serviceAccountName, label: "Uses" });
    }
    for (const vol of spec.volumes ?? []) {
      if (vol.configMap?.name) {
        refs.push({ kind: "ConfigMap", namespace, name: vol.configMap.name, label: "Mounts" });
      }
      if (vol.secret?.secretName) {
        refs.push({ kind: "Secret", namespace, name: vol.secret.secretName, label: "Mounts" });
      }
      if (vol.persistentVolumeClaim?.claimName) {
        refs.push({
          kind: "PersistentVolumeClaim",
          namespace,
          name: vol.persistentVolumeClaim.claimName,
          label: "Mounts",
        });
      }
    }
    for (const container of [...(spec.containers ?? []), ...(spec.initContainers ?? [])]) {
      for (const ef of container.envFrom ?? []) {
        if (ef.configMapRef?.name) {
          refs.push({ kind: "ConfigMap", namespace, name: ef.configMapRef.name, label: "envFrom" });
        }
        if (ef.secretRef?.name) {
          refs.push({ kind: "Secret", namespace, name: ef.secretRef.name, label: "envFrom" });
        }
      }
      for (const env of container.env ?? []) {
        const cmRef = env.valueFrom?.configMapKeyRef;
        const secretRef = env.valueFrom?.secretKeyRef;
        if (cmRef?.name) refs.push({ kind: "ConfigMap", namespace, name: cmRef.name, label: "env" });
        if (secretRef?.name) refs.push({ kind: "Secret", namespace, name: secretRef.name, label: "env" });
      }
    }
  }

  if (kind === "PersistentVolumeClaim" && obj?.spec?.volumeName) {
    refs.push({ kind: "PersistentVolume", name: obj.spec.volumeName, label: "Bound to" });
  }

  // The reverse of the above: a bound PV carries a direct reference back to its claim, so
  // this doesn't need a cluster-wide PVC scan.
  if (kind === "PersistentVolume" && obj?.spec?.claimRef?.name) {
    refs.push({
      kind: "PersistentVolumeClaim",
      namespace: obj.spec.claimRef.namespace,
      name: obj.spec.claimRef.name,
      label: "Bound to",
    });
  }

  return dedupeRefs(refs);
}

/**
 * Child kinds discoverable by listing the child kind in-namespace and filtering by
 * `ownerReferences[].uid` client-side (mirrors the DaemonSet-pod filter already in
 * DrainNodeButton.vue). Doesn't include Service->Pod (label-selector based, resolved
 * server-side instead) or PV<->PVC (a direct ref already covered by `getParentRefs`).
 */
export const OWNED_CHILD_KINDS: Record<string, string[]> = {
  Deployment: ["ReplicaSet"],
  ReplicaSet: ["Pod"],
  StatefulSet: ["Pod"],
  DaemonSet: ["Pod"],
  Job: ["Pod"],
  CronJob: ["Job"],
};
