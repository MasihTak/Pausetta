import { beforeEach, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { appWindow, cursorPosition, getVersion, invoke, listen, openUrl } from "./setup.js";

/** The shape Rust's `get_settings` returns — see src-tauri/src/settings/model.rs. */
export function makeSettings(overrides = {}) {
  return {
    categories: {
      eye: { enabled: true, intervalMinutes: 20 },
      posture: { enabled: true, intervalMinutes: 30 },
      movement: { enabled: true, intervalMinutes: 45 },
      hydration: { enabled: false, intervalMinutes: 60 },
    },
    quietHours: { enabled: true, start: "22:00", end: "08:00" },
    idlePause: { enabled: true, minutes: 5 },
    launchOnLogin: false,
    soundEnabled: false,
    pause: null,
    onboardingCompleted: true,
    ...overrides,
  };
}

/** Resets Pinia and the Tauri mocks to a working default before every test. */
export function useFreshApp() {
  beforeEach(() => {
    vi.clearAllMocks();
    setActivePinia(createPinia());
    invoke.mockResolvedValue(makeSettings());
    listen.mockResolvedValue(vi.fn());
    getVersion.mockResolvedValue("1.2.3");
    openUrl.mockResolvedValue(undefined);
    cursorPosition.mockResolvedValue({ x: 0, y: 0 });
    appWindow.innerPosition.mockResolvedValue({ x: 0, y: 0 });
    appWindow.scaleFactor.mockResolvedValue(1);
    appWindow.setIgnoreCursorEvents.mockResolvedValue(undefined);
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(console, "warn").mockImplementation(() => {});
  });
}
