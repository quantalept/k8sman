/** Route to a resource's detail page. Pod and Node have bespoke detail views (with
 * exec/logs/cordon/etc.) instead of the generic `/resources/:kind/:name` page. */
export function resourcePath(kind: string, namespace: string | undefined, name: string): string {
  if (kind === "Pod") return `/pods/${namespace}/${name}`;
  if (kind === "Node") return `/nodes/${name}`;
  const query = namespace ? `?ns=${encodeURIComponent(namespace)}` : "";
  return `/resources/${kind}/${name}${query}`;
}
