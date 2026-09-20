import { describe, expect, it } from "vitest";
import {
  CATEGORIES,
  CATEGORY_BY_KEY,
  HALF_HOUR_TIME_OPTIONS,
  IDLE_OPTIONS,
  INTERVAL_OPTIONS,
} from "../src/constants/categories.js";
import { ONBOARDING_SLIDES } from "../src/constants/onboarding.js";

describe("categories", () => {
  it("covers the four reminder categories Rust knows about", () => {
    expect(CATEGORIES.map((category) => category.key)).toEqual([
      "eye",
      "posture",
      "movement",
      "hydration",
    ]);
  });

  it("gives every category toast copy", () => {
    for (const category of CATEGORIES) {
      expect(category.label).toBeTruthy();
      expect(category.description).toBeTruthy();
      expect(category.toast).toMatchObject({
        body: expect.any(String),
        cue: expect.any(String),
        action: expect.any(String),
      });
    }
  });

  it("indexes categories by key", () => {
    expect(Object.keys(CATEGORY_BY_KEY)).toHaveLength(CATEGORIES.length);
    expect(CATEGORY_BY_KEY.eye).toBe(CATEGORIES[0]);
  });

  // These must stay in step with INTERVAL_OPTIONS_MINUTES / IDLE_OPTIONS_MINUTES in Rust,
  // or the frontend can offer a value `Settings::validate` rejects.
  it("offers the same interval and idle minutes as the Rust model", () => {
    expect(INTERVAL_OPTIONS).toEqual([
      { value: 15, label: "15 min" },
      { value: 20, label: "20 min" },
      { value: 30, label: "30 min" },
      { value: 45, label: "45 min" },
      { value: 60, label: "60 min" },
    ]);
    expect(IDLE_OPTIONS.map((option) => option.value)).toEqual([3, 5, 10, 15, 30]);
  });
});

describe("half-hour time options", () => {
  it("covers a full day on the half hour", () => {
    expect(HALF_HOUR_TIME_OPTIONS).toHaveLength(48);
    expect(HALF_HOUR_TIME_OPTIONS.map((option) => option.value)).toContain("22:00");
  });

  it("stores 24h values and shows 12h labels", () => {
    const labelFor = (value) =>
      HALF_HOUR_TIME_OPTIONS.find((option) => option.value === value).label;

    expect(labelFor("00:00")).toBe("12:00 AM");
    expect(labelFor("00:30")).toBe("12:30 AM");
    expect(labelFor("09:30")).toBe("9:30 AM");
    expect(labelFor("12:00")).toBe("12:00 PM");
    expect(labelFor("13:00")).toBe("1:00 PM");
    expect(labelFor("23:30")).toBe("11:30 PM");
  });
});

describe("onboarding slides", () => {
  it("opens with a welcome and then one slide per category", () => {
    expect(ONBOARDING_SLIDES.map((slide) => slide.key)).toEqual([
      "welcome",
      ...CATEGORIES.map((category) => category.key),
    ]);
  });

  // science-research.md asks for the caveat on every evidence claim; only the welcome is exempt.
  it("carries an evidence note on every category slide", () => {
    for (const slide of ONBOARDING_SLIDES.slice(1)) {
      expect(slide.note).toBeTruthy();
    }
    expect(ONBOARDING_SLIDES[0].note).toBeUndefined();
  });
});
