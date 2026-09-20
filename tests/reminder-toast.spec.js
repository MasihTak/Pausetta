import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import ReminderToast from "../src/components/ReminderToast.vue";
import { CATEGORY_BY_KEY } from "../src/constants/categories.js";
import { invoke, listen } from "./setup.js";
import { useFreshApp } from "./helpers.js";

useFreshApp();

// Rust injects both globals when it creates the reminder window (reminder_window.rs).
beforeEach(() => {
  window.__PAUSETTA_REMINDER_CATEGORY__ = "eye";
  window.__PAUSETTA_REMINDER_ANCHOR__ = "bottom";
});

afterEach(() => {
  vi.useRealTimers();
  delete window.__PAUSETTA_REMINDER_CATEGORY__;
  delete window.__PAUSETTA_REMINDER_ANCHOR__;
});

async function mountToast() {
  let emitReminderChanged;
  const stopListening = vi.fn();
  listen.mockImplementation((_event, handler) => {
    emitReminderChanged = handler;
    return Promise.resolve(stopListening);
  });

  const toast = mount(ReminderToast);
  await flushPromises();
  return { toast, stopListening, emit: (payload) => emitReminderChanged({ payload }) };
}

describe("ReminderToast", () => {
  it("shows the injected category's copy", async () => {
    const { toast } = await mountToast();

    expect(toast.find(".toast__title").text()).toBe("Eye care");
    expect(toast.find(".toast__body").text()).toBe(CATEGORY_BY_KEY.eye.toast.body);
    expect(toast.find(".toast__cue-text").text()).toBe(CATEGORY_BY_KEY.eye.toast.cue);
    expect(toast.find(".toast__primary").text()).toBe(CATEGORY_BY_KEY.eye.toast.action);
  });

  it("shows the category's interval once settings arrive", async () => {
    const { toast } = await mountToast();

    expect(toast.find(".toast__chip").text()).toBe("Every 20 min");
  });

  it("renders without a chip when settings can't be loaded", async () => {
    invoke.mockRejectedValue(new Error("no database"));
    const { toast } = await mountToast();

    expect(toast.find(".toast__chip").exists()).toBe(false);
    expect(toast.find(".toast__title").text()).toBe("Eye care");
  });

  it.each([
    ["eye", ".cue-ring"],
    ["posture", ".cue-posture"],
    ["movement", ".cue-bars"],
    ["hydration", ".cue-glass"],
  ])("draws the %s cue", async (category, selector) => {
    window.__PAUSETTA_REMINDER_CATEGORY__ = category;
    const { toast } = await mountToast();

    expect(toast.find(selector).exists()).toBe(true);
  });

  it("anchors to the top when Rust says the notification area is up there", async () => {
    window.__PAUSETTA_REMINDER_ANCHOR__ = "top";
    const { toast } = await mountToast();

    expect(toast.find(".toast-stage").classes()).toContain("is-top");
  });

  it("asks Rust to close the window on the primary action", async () => {
    const { toast } = await mountToast();

    await toast.find(".toast__primary").trigger("click");

    expect(invoke).toHaveBeenCalledWith("close_reminder");
  });

  it("asks Rust to close the window on the close button", async () => {
    const { toast } = await mountToast();

    await toast.find(".toast__close").trigger("click");

    expect(invoke).toHaveBeenCalledWith("close_reminder");
  });

  it("asks Rust to move the next trigger on snooze", async () => {
    const { toast } = await mountToast();

    await toast.find(".toast__ghost").trigger("click");

    expect(invoke).toHaveBeenCalledWith("snooze_reminder", { category: "eye" });
  });

  it("logs, rather than throws, when Rust rejects a close or a snooze", async () => {
    const { toast } = await mountToast();
    invoke.mockRejectedValue(new Error("window gone"));

    await toast.find(".toast__primary").trigger("click");
    await toast.find(".toast__ghost").trigger("click");
    await flushPromises();

    expect(console.error).toHaveBeenCalledTimes(2);
  });

  it("closes itself if nobody acts on it", async () => {
    vi.useFakeTimers();
    const { toast } = await mountToast();

    expect(invoke).not.toHaveBeenCalledWith("close_reminder");
    vi.advanceTimersByTime(14_000);

    expect(invoke).toHaveBeenCalledWith("close_reminder");
    toast.unmount();
  });

  it("swaps category and restarts the countdown when Rust reuses the window", async () => {
    vi.useFakeTimers();
    const { toast, emit } = await mountToast();

    vi.advanceTimersByTime(13_000);
    emit("hydration");
    await toast.vm.$nextTick();

    expect(toast.find(".toast__title").text()).toBe("Hydration");
    expect(toast.find(".toast__chip").text()).toBe("Every 60 min");

    // The old timer would have fired here; the new reminder gets its own full window.
    vi.advanceTimersByTime(13_000);
    expect(invoke).not.toHaveBeenCalledWith("close_reminder");

    vi.advanceTimersByTime(1_000);
    expect(invoke).toHaveBeenCalledWith("close_reminder");
    toast.unmount();
  });

  it("drops its timer and listener when unmounted", async () => {
    vi.useFakeTimers();
    const { toast, stopListening } = await mountToast();

    toast.unmount();
    vi.advanceTimersByTime(20_000);

    expect(stopListening).toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalledWith("close_reminder");
  });
});
