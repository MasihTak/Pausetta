import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";

// A thin mirror of the settings the Rust core owns. It never decides anything itself:
// every change is sent to Rust, and Rust's response replaces the local copy.
export const useSettingsStore = defineStore("settings", () => {
  const settings = ref(null);
  // Rust's reason for the last rejected change; without it the setting just snaps back.
  const saveError = ref("");

  async function load() {
    settings.value = await invoke("get_settings");
  }

  async function replaceWith(command, args) {
    try {
      settings.value = await invoke(command, args);
      saveError.value = "";
    } catch (error) {
      console.error(`[pausetta] ${command} failed`, error);
      saveError.value = String(error);
      try {
        await load();
      } catch (reloadError) {
        console.error("[pausetta] could not reload settings after a failed update", reloadError);
      }
    }
  }

  /** Applies `mutate` optimistically, then persists; reloads from Rust if it's rejected. */
  async function update(mutate) {
    mutate(settings.value);
    await replaceWith("update_settings", { settings: settings.value });
  }

  const pause = (kind) => replaceWith("pause_reminders", { kind });
  const resume = () => replaceWith("resume_reminders");

  function dismissSaveError() {
    saveError.value = "";
  }

  return { settings, saveError, load, update, pause, resume, dismissSaveError };
});

/** A writable computed over one setting, for use with v-model. */
export function useSettingModel(readSetting, writeSetting) {
  const store = useSettingsStore();
  return computed({
    get: () => readSetting(store.settings),
    set: (value) => store.update((settings) => writeSetting(settings, value)),
  });
}
