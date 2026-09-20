import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import AppMark from "../src/components/AppMark.vue";
import CategoryIcon from "../src/components/CategoryIcon.vue";
import CategoryRow from "../src/components/CategoryRow.vue";
import OnboardingScene from "../src/components/OnboardingScene.vue";
import SelectField from "../src/components/SelectField.vue";
import ToggleSwitch from "../src/components/ToggleSwitch.vue";
import { CATEGORIES } from "../src/constants/categories.js";
import { useSettingsStore } from "../src/stores/settings.js";
import { invoke } from "./setup.js";
import { makeSettings, useFreshApp } from "./helpers.js";

useFreshApp();

describe("ToggleSwitch", () => {
  it("exposes its state to assistive tech", () => {
    const toggle = mount(ToggleSwitch, {
      props: { modelValue: true, label: "Hydration" },
    });

    expect(toggle.attributes("role")).toBe("switch");
    expect(toggle.attributes("aria-checked")).toBe("true");
    expect(toggle.attributes("aria-label")).toBe("Hydration");
    expect(toggle.classes()).toContain("is-on");
  });

  it("emits the flipped value on click", async () => {
    const toggle = mount(ToggleSwitch, {
      props: { modelValue: false, label: "Hydration" },
    });

    await toggle.trigger("click");

    expect(toggle.emitted("update:modelValue")).toEqual([[true]]);
  });
});

describe("SelectField", () => {
  const options = [
    { value: 15, label: "15 min" },
    { value: 30, label: "30 min" },
  ];

  it("renders every option and labels the native select", () => {
    const field = mount(SelectField, {
      props: { modelValue: 15, options, label: "Eye care interval" },
    });

    expect(field.findAll("option").map((option) => option.text())).toEqual(["15 min", "30 min"]);
    expect(field.find("select").attributes("aria-label")).toBe("Eye care interval");
    expect(field.find("select").element.value).toBe("15");
  });

  it("emits the picked option's value", async () => {
    const field = mount(SelectField, {
      props: { modelValue: 15, options, label: "Eye care interval" },
    });

    await field.find("select").setValue("30");

    expect(field.emitted("update:modelValue")).toEqual([[30]]);
  });

  it("can be disabled and made compact", () => {
    const field = mount(SelectField, {
      props: { modelValue: 15, options, label: "Eye care interval", disabled: true, isCompact: true },
    });

    expect(field.find("select").attributes("disabled")).toBeDefined();
    expect(field.classes()).toContain("is-compact");
  });
});

describe("CategoryIcon", () => {
  it.each(CATEGORIES.map((category) => category.key))("draws the %s glyph", (key) => {
    const icon = mount(CategoryIcon, { props: { category: key, size: 20 } });

    expect(icon.attributes("width")).toBe("20");
    expect(icon.element.querySelectorAll("path, circle").length).toBeGreaterThan(0);
  });

  it("draws nothing for an unknown category rather than throwing", () => {
    const icon = mount(CategoryIcon, { props: { category: "nope" } });

    expect(icon.element.querySelectorAll("path, circle")).toHaveLength(0);
  });
});

describe("AppMark", () => {
  it("defaults to 28px and is hidden from assistive tech", () => {
    // The template leads with a comment, so the svg is not the wrapper's root node.
    const svg = mount(AppMark).find("svg");

    expect(svg.attributes("width")).toBe("28");
    expect(svg.attributes("aria-hidden")).toBe("true");
  });
});

describe("OnboardingScene", () => {
  // All scenes mount at once in the carousel, so their clip-path ids must not collide.
  it.each(["welcome", "eye", "posture", "movement", "hydration"])(
    "draws the %s vignette with namespaced clip ids",
    (category) => {
      const scene = mount(OnboardingScene, { props: { category } });

      expect(scene.element.querySelectorAll("path, circle, rect").length).toBeGreaterThan(1);
      for (const clip of scene.element.querySelectorAll("clipPath")) {
        expect(clip.id).toContain(category);
      }
    },
  );
});

describe("CategoryRow", () => {
  const mountRow = async (category = CATEGORIES[0]) => {
    const store = useSettingsStore();
    await store.load();
    return mount(CategoryRow, { props: { category } });
  };

  it("shows the category's label, description and current interval", async () => {
    const row = await mountRow();

    expect(row.text()).toContain("Eye care");
    expect(row.text()).toContain(CATEGORIES[0].description);
    expect(row.find("select").element.value).toBe("20");
    expect(row.findComponent(ToggleSwitch).props("modelValue")).toBe(true);
  });

  it("dims and disables the interval while the category is off", async () => {
    const row = await mountRow(CATEGORIES[3]);

    expect(row.find(".category-detail").classes()).toContain("is-dimmed");
    expect(row.find("select").attributes("disabled")).toBeDefined();
  });

  it("persists a toggle through the store", async () => {
    const row = await mountRow();

    invoke.mockResolvedValueOnce(makeSettings());
    await row.findComponent(ToggleSwitch).trigger("click");

    expect(invoke).toHaveBeenLastCalledWith("update_settings", {
      settings: expect.objectContaining({
        categories: expect.objectContaining({ eye: { enabled: false, intervalMinutes: 20 } }),
      }),
    });
  });

  it("persists an interval change through the store", async () => {
    const row = await mountRow();

    invoke.mockResolvedValueOnce(makeSettings());
    await row.find("select").setValue("45");

    expect(invoke).toHaveBeenLastCalledWith("update_settings", {
      settings: expect.objectContaining({
        categories: expect.objectContaining({ eye: { enabled: true, intervalMinutes: 45 } }),
      }),
    });
  });
});
