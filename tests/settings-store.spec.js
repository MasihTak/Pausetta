import { describe, expect, it } from "vitest";
import { useSettingModel, useSettingsStore } from "../src/stores/settings.js";
import { invoke } from "./setup.js";
import { makeSettings, useFreshApp } from "./helpers.js";

useFreshApp();

describe("useSettingsStore", () => {
  it("starts empty and fills from Rust on load", async () => {
    const store = useSettingsStore();
    expect(store.settings).toBeNull();

    await store.load();

    expect(invoke).toHaveBeenCalledWith("get_settings");
    expect(store.settings.categories.eye.intervalMinutes).toBe(20);
  });

  it("lets load's failure reach the caller", async () => {
    const store = useSettingsStore();
    invoke.mockRejectedValueOnce(new Error("no database"));

    await expect(store.load()).rejects.toThrow("no database");
  });

  it("sends the mutated settings to Rust and keeps Rust's answer", async () => {
    const store = useSettingsStore();
    await store.load();

    const fromRust = makeSettings({ soundEnabled: true });
    invoke.mockResolvedValueOnce(fromRust);
    await store.update((settings) => (settings.soundEnabled = true));

    expect(invoke).toHaveBeenLastCalledWith("update_settings", {
      settings: expect.objectContaining({ soundEnabled: true }),
    });
    expect(store.settings).toEqual(fromRust);
  });

  it("reloads from Rust when an update is rejected, discarding the optimistic edit", async () => {
    const store = useSettingsStore();
    await store.load();

    invoke.mockRejectedValueOnce(new Error("invalid interval"));
    invoke.mockResolvedValueOnce(makeSettings());
    await store.update((settings) => (settings.categories.eye.intervalMinutes = 7));

    expect(invoke).toHaveBeenLastCalledWith("get_settings");
    expect(store.settings.categories.eye.intervalMinutes).toBe(20);
  });

  it("survives a reload that also fails", async () => {
    const store = useSettingsStore();
    await store.load();

    invoke.mockRejectedValue(new Error("still broken"));
    await expect(store.update((settings) => (settings.soundEnabled = true))).resolves.toBeUndefined();
  });

  it("asks Rust to pause and to resume", async () => {
    const store = useSettingsStore();
    const paused = makeSettings({ pause: { kind: "today", until: "2026-09-19T22:00:00Z" } });

    invoke.mockResolvedValueOnce(paused);
    await store.pause("today");
    expect(invoke).toHaveBeenLastCalledWith("pause_reminders", { kind: "today" });
    expect(store.settings.pause.kind).toBe("today");

    invoke.mockResolvedValueOnce(makeSettings());
    await store.resume();
    expect(invoke).toHaveBeenLastCalledWith("resume_reminders", undefined);
    expect(store.settings.pause).toBeNull();
  });
});

describe("useSettingModel", () => {
  it("reads through to the store", async () => {
    const store = useSettingsStore();
    await store.load();

    const soundEnabled = useSettingModel(
      (settings) => settings.soundEnabled,
      (settings, value) => (settings.soundEnabled = value),
    );

    expect(soundEnabled.value).toBe(false);
  });

  it("writes through the store's update, so Rust persists it", async () => {
    const store = useSettingsStore();
    await store.load();

    const idleMinutes = useSettingModel(
      (settings) => settings.idlePause.minutes,
      (settings, value) => (settings.idlePause.minutes = value),
    );

    invoke.mockResolvedValueOnce(makeSettings({ idlePause: { enabled: true, minutes: 15 } }));
    idleMinutes.value = 15;
    await Promise.resolve();

    expect(invoke).toHaveBeenLastCalledWith("update_settings", {
      settings: expect.objectContaining({ idlePause: { enabled: true, minutes: 15 } }),
    });
  });
});
