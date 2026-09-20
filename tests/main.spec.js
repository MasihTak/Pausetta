import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { listen } from "./setup.js";
import { useFreshApp } from "./helpers.js";

useFreshApp();

beforeEach(() => {
  vi.resetModules();
  listen.mockResolvedValue(vi.fn());
  document.body.innerHTML = '<div id="app"></div>';
  document.documentElement.className = "";
});

afterEach(() => {
  delete window.__PAUSETTA_REMINDER_CATEGORY__;
});

describe("main entry", () => {
  it("mounts the Settings app when Rust injected no reminder category", async () => {
    await import("../src/main.js");

    expect(document.documentElement.classList.contains("is-reminder")).toBe(false);
    expect(document.querySelector(".app-footer")).not.toBeNull();
  });

  // Rust injects this global only into the reminder window (reminder_window.rs).
  it("mounts the toast, and flags the document, in the reminder window", async () => {
    window.__PAUSETTA_REMINDER_CATEGORY__ = "movement";
    await import("../src/main.js");

    expect(document.documentElement.classList.contains("is-reminder")).toBe(true);
    expect(document.querySelector(".toast__title").textContent.trim()).toBe("Movement break");
    expect(document.querySelector(".app-footer")).toBeNull();
  });
});
