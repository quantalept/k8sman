import { invoke } from "@tauri-apps/api/core";

export interface RuleView {
  apiGroups: string[];
  resources: string[];
  resourceNames: string[];
  verbs: string[];
  nonResourceUrls: string[];
}

export interface BindingRules {
  bindingKind: string;
  bindingName: string;
  bindingNamespace?: string;
  roleKind: string;
  roleName: string;
  rules: RuleView[];
}

export function getSubjectRules(
  contextName: string,
  kind: string,
  namespace: string | undefined,
  name: string,
): Promise<BindingRules[]> {
  return invoke("get_subject_rules", { contextName, kind, namespace, name });
}
