/**
 * Heuristic for the dual-purpose namespace-filter-row text input: real Kubernetes label
 * selectors always use `key=value` (or `key!=value`) pairs, so anything containing `=`
 * is treated as a selector sent to the API; anything else is treated as a plain-text
 * client-side name search instead.
 */
export function looksLikeLabelSelector(text: string): boolean {
  return text.includes("=");
}
