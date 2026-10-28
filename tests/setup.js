import { vi } from "vitest";

// What Rust injects into every window (window_script.rs); the constants read it on import.
window.__PAUSETTA_OPTIONS__ = {
  intervalMinutes: [15, 20, 30, 45, 60],
  idleMinutes: [3, 5, 10, 15, 30],
  snoozeMinutes: 10,
};

// One mock per Tauri module the frontend imports. Tests import these same fns to
// stub responses and assert calls — there is no Tauri runtime under jsdom.
export const invoke = vi.fn();
export const listen = vi.fn();
export const getVersion = vi.fn();
export const openUrl = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args) => invoke(...args) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: (...args) => listen(...args) }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion: (...args) => getVersion(...args) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: (...args) => openUrl(...args) }));
