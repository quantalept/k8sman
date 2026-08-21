/**
 * Evaluates the small subset of Kubernetes' JSONPath dialect used by CRD
 * `additionalPrinterColumns` (the same paths `kubectl get <crd>` renders). Handles plain
 * field access (`.spec.replicas`), array indices (`.spec.containers[0].name`), and the
 * `[?(@.field=="value")]` filter K8s uses pervasively for status conditions
 * (`.status.conditions[?(@.type=="Ready")].status`) - NOT full JSONPath (no slices,
 * unions, wildcards, or recursive descent).
 */
export function evalPrinterColumnPath(obj: any, path: string): unknown {
  if (!path.startsWith(".")) return undefined;
  let current: unknown = obj;
  for (const token of splitPath(path.slice(1))) {
    for (const step of parseToken(token)) {
      if (current === undefined || current === null) return undefined;
      current = applyStep(current, step);
    }
  }
  return current;
}

type Step =
  | { kind: "field"; name: string }
  | { kind: "index"; index: number }
  | { kind: "filter"; field: string; value: string };

/** Splits on `.` but keeps bracket expressions (which may contain their own `.`) intact. */
function splitPath(path: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let current = "";
  for (const ch of path) {
    if (ch === "[") depth++;
    if (ch === "]") depth--;
    if (ch === "." && depth === 0) {
      if (current) parts.push(current);
      current = "";
    } else {
      current += ch;
    }
  }
  if (current) parts.push(current);
  return parts;
}

function parseToken(token: string): Step[] {
  const steps: Step[] = [];
  const match = token.match(/^([^[]*)(\[.*\])?$/);
  const fieldName = match?.[1] ?? token;
  const bracket = match?.[2];

  if (fieldName) steps.push({ kind: "field", name: fieldName });

  if (bracket) {
    const inner = bracket.slice(1, -1);
    if (inner.startsWith("?(") && inner.endsWith(")")) {
      const expr = inner.slice(2, -1);
      const filterMatch = expr.match(/^@\.(\w+)==["'](.*)["']$/);
      if (filterMatch) {
        steps.push({ kind: "filter", field: filterMatch[1], value: filterMatch[2] });
      }
    } else if (/^\d+$/.test(inner)) {
      steps.push({ kind: "index", index: parseInt(inner, 10) });
    }
  }

  return steps;
}

function applyStep(current: unknown, step: Step): unknown {
  if (step.kind === "field") {
    return (current as Record<string, unknown>)?.[step.name];
  }
  if (step.kind === "index") {
    return Array.isArray(current) ? current[step.index] : undefined;
  }
  return Array.isArray(current)
    ? current.find((item) => item?.[step.field] === step.value)
    : undefined;
}
