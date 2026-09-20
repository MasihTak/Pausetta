import { afterEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import PauseMenu from "../src/components/PauseMenu.vue";
import { useSettingsStore } from "../src/stores/settings.js";
import { invoke } from "./setup.js";
import { makeSettings, useFreshApp } from "./helpers.js";

useFreshApp();
afterEach(() => vi.useRealTimers());

const NOW = new Date("2026-09-19T12:00:00Z");
const minutesFromNow = (minutes) => new Date(NOW.getTime() + minutes * 60_000).toISOString();

async function mountMenu(pause = null) {
  invoke.mockResolvedValueOnce(makeSettings({ pause }));
  const store = useSettingsStore();
  await store.load();
  return { store, menu: mount(PauseMenu, { attachTo: document.body }) };
}

describe("PauseMenu button", () => {
  it("reads 'Pause' while reminders are running", async () => {
    const { menu } = await mountMenu();

    expect(menu.find(".pause__button").text()).toBe("Pause");
    expect(menu.find(".pause__button").classes()).not.toContain("is-paused");
  });

  it("names the remaining time for a timed pause", async () => {
    vi.useFakeTimers({ now: NOW });
    const { menu } = await mountMenu({ kind: "oneHour", until: minutesFromNow(45) });

    expect(menu.find(".pause__button").text()).toBe("Paused · 45 min left");
    expect(menu.find(".pause__button").classes()).toContain("is-paused");
  });

  it.each([
    [90, "1 hr 30 min"],
    [60, "1 hr"],
    [1, "1 min"],
  ])("formats %i minutes left as '%s'", async (minutes, expected) => {
    vi.useFakeTimers({ now: NOW });
    const { menu } = await mountMenu({ kind: "oneHour", until: minutesFromNow(minutes) });

    expect(menu.find(".pause__button").text()).toBe(`Paused · ${expected} left`);
  });

  it("says 'Paused for today' without a countdown", async () => {
    vi.useFakeTimers({ now: NOW });
    const { menu } = await mountMenu({ kind: "today", until: minutesFromNow(600) });

    expect(menu.find(".pause__button").text()).toBe("Paused for today");
  });

  it("treats an elapsed pause as running again", async () => {
    vi.useFakeTimers({ now: NOW });
    const { menu } = await mountMenu({ kind: "oneHour", until: minutesFromNow(-1) });

    expect(menu.find(".pause__button").text()).toBe("Pause");
  });

  // Without the watch on store.settings.pause, a fresh pause is measured against a
  // clock up to 30s stale and reads "1 hr 1 min left".
  it("resets its clock when a new pause arrives, so 1 hour reads as 1 hour", async () => {
    vi.useFakeTimers({ now: NOW });
    const { store, menu } = await mountMenu();

    vi.advanceTimersByTime(29_000);
    store.settings.pause = { kind: "oneHour", until: new Date(Date.now() + 3_600_000).toISOString() };
    await menu.vm.$nextTick();

    expect(menu.find(".pause__button").text()).toBe("Paused · 1 hr left");
  });

  it("keeps the countdown ticking down", async () => {
    vi.useFakeTimers({ now: NOW });
    const { menu } = await mountMenu({ kind: "oneHour", until: minutesFromNow(45) });

    vi.advanceTimersByTime(120_000);
    await menu.vm.$nextTick();

    expect(menu.find(".pause__button").text()).toBe("Paused · 43 min left");
  });
});

describe("PauseMenu menu", () => {
  it("stays closed until asked", async () => {
    const { menu } = await mountMenu();

    expect(menu.find('[role="menu"]').exists()).toBe(false);
    expect(menu.find(".pause__button").attributes("aria-expanded")).toBe("false");
  });

  it("opens on click and focuses the first item", async () => {
    const { menu } = await mountMenu();

    await menu.find(".pause__button").trigger("click");
    await menu.vm.$nextTick();

    expect(menu.find('[role="menu"]').exists()).toBe(true);
    expect(menu.find(".pause__button").attributes("aria-expanded")).toBe("true");
    expect(document.activeElement.textContent).toContain("Pause for 1 hour");
    menu.unmount();
  });

  it("closes on a second click", async () => {
    const { menu } = await mountMenu();

    await menu.find(".pause__button").trigger("click");
    await menu.find(".pause__button").trigger("click");

    expect(menu.find('[role="menu"]').exists()).toBe(false);
  });

  it("closes on a backdrop click", async () => {
    const { menu } = await mountMenu();

    await menu.find(".pause__button").trigger("click");
    await menu.find(".pause__backdrop").trigger("click");

    expect(menu.find('[role="menu"]').exists()).toBe(false);
  });

  it("closes on Escape", async () => {
    const { menu } = await mountMenu();
    await menu.find(".pause__button").trigger("click");

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    await menu.vm.$nextTick();

    expect(menu.find('[role="menu"]').exists()).toBe(false);
  });

  it("ignores other keys", async () => {
    const { menu } = await mountMenu();
    await menu.find(".pause__button").trigger("click");

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));
    await menu.vm.$nextTick();

    expect(menu.find('[role="menu"]').exists()).toBe(true);
  });

  it("offers no Resume while reminders are running", async () => {
    const { menu } = await mountMenu();
    await menu.find(".pause__button").trigger("click");

    expect(menu.findAll('[role="menuitem"]').map((item) => item.text())).toEqual([
      "Pause for 1 hour",
      "Pause for today",
    ]);
  });

  it.each([
    [0, "oneHour"],
    [1, "today"],
  ])("sends pause kind '%s' to Rust and closes", async (index, kind) => {
    const { menu } = await mountMenu();
    await menu.find(".pause__button").trigger("click");

    invoke.mockResolvedValueOnce(makeSettings());
    await menu.findAll('[role="menuitem"]')[index].trigger("click");

    expect(invoke).toHaveBeenLastCalledWith("pause_reminders", { kind });
    expect(menu.find('[role="menu"]').exists()).toBe(false);
  });

  it("offers Resume once paused, and sends it to Rust", async () => {
    vi.useFakeTimers({ now: NOW });
    const { menu } = await mountMenu({ kind: "today", until: minutesFromNow(600) });
    await menu.find(".pause__button").trigger("click");

    const resume = menu.findAll('[role="menuitem"]').at(-1);
    expect(resume.text()).toBe("Resume");

    invoke.mockResolvedValueOnce(makeSettings());
    await resume.trigger("click");

    expect(invoke).toHaveBeenLastCalledWith("resume_reminders", undefined);
  });

  it("drops its clock and key listener when unmounted", async () => {
    vi.useFakeTimers({ now: NOW });
    const clearInterval = vi.spyOn(globalThis, "clearInterval");
    const removeListener = vi.spyOn(window, "removeEventListener");
    const { menu } = await mountMenu();

    menu.unmount();

    expect(clearInterval).toHaveBeenCalled();
    expect(removeListener).toHaveBeenCalledWith("keydown", expect.any(Function));
  });
});
