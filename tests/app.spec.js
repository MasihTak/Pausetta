import { afterEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import App from "../src/App.vue";
import AboutView from "../src/views/AboutView.vue";
import OnboardingView from "../src/views/OnboardingView.vue";
import SettingsView from "../src/views/SettingsView.vue";
import PauseMenu from "../src/components/PauseMenu.vue";
import { invoke, listen, openUrl } from "./setup.js";
import { makeSettings, useFreshApp } from "./helpers.js";
import { useSettingsStore } from "../src/stores/settings.js";

useFreshApp();

afterEach(() => {
  delete window.__PAUSETTA_SCREEN__;
});

/** Mounts App and hands back the `show-screen` / `settings-changed` handlers Rust would call. */
async function mountApp() {
  const handlers = {};
  const stopListening = [vi.fn(), vi.fn()];
  let listenCount = 0;
  listen.mockImplementation((event, handler) => {
    handlers[event] = handler;
    return Promise.resolve(stopListening[listenCount++]);
  });

  const app = mount(App);
  await flushPromises();
  return { app, handlers, stopListening };
}

describe("App screens", () => {
  it("loads settings from Rust on mount", async () => {
    await mountApp();

    expect(invoke).toHaveBeenCalledWith("get_settings");
  });

  it("opens on Settings by default", async () => {
    const { app } = await mountApp();

    expect(app.findComponent(SettingsView).exists()).toBe(true);
    expect(app.findComponent(PauseMenu).exists()).toBe(true);
  });

  // Rust injects the screen when it creates the window (settings_window.rs).
  it("opens on the screen Rust injected", async () => {
    window.__PAUSETTA_SCREEN__ = "about";
    const { app } = await mountApp();

    expect(app.findComponent(AboutView).exists()).toBe(true);
    expect(app.find(".back-button").exists()).toBe(true);
  });

  it("navigates to About and back", async () => {
    const { app } = await mountApp();

    await app.find(".icon-button").trigger("click");
    expect(app.findComponent(AboutView).exists()).toBe(true);

    await app.find(".back-button").trigger("click");
    expect(app.findComponent(SettingsView).exists()).toBe(true);
  });

  // An already-open window is told which screen to show with an event instead.
  it("switches screen when Rust sends show-screen", async () => {
    const { app, handlers } = await mountApp();

    handlers["show-screen"]({ payload: "about" });
    await app.vm.$nextTick();

    expect(app.findComponent(AboutView).exists()).toBe(true);
  });

  it("reloads settings when the tray changes them behind this window's back", async () => {
    const { handlers } = await mountApp();
    invoke.mockClear();

    handlers["settings-changed"]();
    await flushPromises();

    expect(invoke).toHaveBeenCalledWith("get_settings");
  });

  it("logs a failed background reload instead of breaking the window", async () => {
    const { handlers } = await mountApp();
    invoke.mockRejectedValueOnce(new Error("locked"));

    handlers["settings-changed"]();
    await flushPromises();

    expect(console.error).toHaveBeenCalled();
  });

  it("shows why settings couldn't load", async () => {
    invoke.mockRejectedValue(new Error("no database"));
    const { app } = await mountApp();

    const alert = app.find('[role="alert"]');
    expect(alert.text()).toContain("no database");
    expect(app.findComponent(SettingsView).exists()).toBe(false);
    // The pause control needs settings, so it stays away too.
    expect(app.findComponent(PauseMenu).exists()).toBe(false);
  });

  it("still lets you reach About when settings failed to load", async () => {
    invoke.mockRejectedValue(new Error("no database"));
    const { app } = await mountApp();

    await app.find(".icon-button").trigger("click");

    expect(app.findComponent(AboutView).exists()).toBe(true);
    expect(app.find('[role="alert"]').exists()).toBe(false);
  });

  it("still loads settings when an event listener fails to register", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    const stopShowScreen = vi.fn();
    listen.mockImplementation((event) =>
      event === "settings-changed"
        ? Promise.reject(new Error("no listener"))
        : Promise.resolve(stopShowScreen),
    );

    const app = mount(App);
    await flushPromises();

    expect(app.findComponent(SettingsView).exists()).toBe(true);
    app.unmount();
    expect(stopShowScreen).toHaveBeenCalledOnce();
  });

  it("shows why a change was rejected until it's dismissed", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    const { app } = await mountApp();
    const store = useSettingsStore();

    invoke.mockRejectedValueOnce("could not register login item");
    await store.update((settings) => (settings.launchOnLogin = true));
    await flushPromises();

    const alert = app.find(".save-error");
    expect(alert.attributes("role")).toBe("alert");
    expect(alert.text()).toContain("could not register login item");

    await app.find(".save-error__dismiss").trigger("click");
    expect(app.find(".save-error").exists()).toBe(false);
  });

  it("drops its event listeners when unmounted", async () => {
    const { app, stopListening } = await mountApp();

    app.unmount();

    expect(stopListening.every((stop) => stop.mock.calls.length === 1)).toBe(true);
  });
});

describe("App onboarding", () => {
  const unfinished = () => makeSettings({ onboardingCompleted: false });

  it("shows onboarding instead of the app until it's completed", async () => {
    invoke.mockResolvedValue(unfinished());
    const { app } = await mountApp();

    expect(app.findComponent(OnboardingView).exists()).toBe(true);
    expect(app.findComponent(SettingsView).exists()).toBe(false);
  });

  it("persists completion in Rust and reveals the app", async () => {
    invoke.mockResolvedValueOnce(unfinished());
    const { app } = await mountApp();

    invoke.mockResolvedValueOnce(makeSettings({ onboardingCompleted: true }));
    app.findComponent(OnboardingView).vm.$emit("finish");
    await flushPromises();

    expect(invoke).toHaveBeenLastCalledWith("update_settings", {
      settings: expect.objectContaining({ onboardingCompleted: true }),
    });
    expect(app.findComponent(SettingsView).exists()).toBe(true);
  });

  it("logs a failed completion save rather than trapping the user", async () => {
    invoke.mockResolvedValueOnce(unfinished());
    const { app } = await mountApp();

    invoke.mockRejectedValueOnce(new Error("disk full"));
    invoke.mockResolvedValueOnce(unfinished());
    app.findComponent(OnboardingView).vm.$emit("finish");
    await flushPromises();

    expect(console.error).toHaveBeenCalled();
  });
});

describe("App footer", () => {
  // The webview itself must not navigate, so Rust opens the link.
  it("hands the author link to Rust instead of navigating", async () => {
    const { app } = await mountApp();
    const link = app.find(".app-footer a");

    expect(link.attributes("href")).toBe("https://masih.dev");
    await link.trigger("click");

    expect(openUrl).toHaveBeenCalledWith("https://masih.dev");
  });

  it("logs a link that couldn't be opened", async () => {
    openUrl.mockRejectedValue(new Error("no opener"));
    const { app } = await mountApp();

    await app.find(".app-footer a").trigger("click");
    await flushPromises();

    expect(console.error).toHaveBeenCalled();
  });
});
