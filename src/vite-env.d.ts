/// <reference types="vite/client" />

/** Injected by vite.config.ts `define`, from package.json. */
declare const __APP_VERSION__: string;

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
