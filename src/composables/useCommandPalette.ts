import { ref } from "vue";

const isOpen = ref(false);

export function useCommandPalette() {
  function open() {
    isOpen.value = true;
  }
  function close() {
    isOpen.value = false;
  }
  function toggle() {
    isOpen.value = !isOpen.value;
  }
  return { isOpen, open, close, toggle };
}

let listenerRegistered = false;

/** Registers the global Cmd/Ctrl+K shortcut. Call once, from App.vue. */
export function registerCommandPaletteShortcut() {
  if (listenerRegistered) return;
  listenerRegistered = true;
  window.addEventListener("keydown", (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      useCommandPalette().toggle();
    }
  });
}
