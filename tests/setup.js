import { vi } from "vitest";

// One mock per Tauri module the frontend imports. Tests import these same fns to
// stub responses and assert calls — there is no Tauri runtime under jsdom.
export const invoke = vi.fn();
export const listen = vi.fn();
export const getVersion = vi.fn();
export const openUrl = vi.fn();
export const cursorPosition = vi.fn();
export const appWindow = {
  innerPosition: vi.fn(),
  scaleFactor: vi.fn(),
  setIgnoreCursorEvents: vi.fn(),
};

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args) => invoke(...args) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: (...args) => listen(...args) }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion: (...args) => getVersion(...args) }));
vi.mock("@tauri-apps/api/window", () => ({
  cursorPosition: (...args) => cursorPosition(...args),
  getCurrentWindow: () => appWindow,
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: (...args) => openUrl(...args) }));
