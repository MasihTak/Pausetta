// Copy is kept calm and non-absolute, per science-research.md: these are sensible habits,
// not proven medical interventions.
export const CATEGORIES = [
  {
    key: "eye",
    label: "Eye care",
    description: "20-20-20 rule — look 20 feet away for 20 seconds.",
    toast: {
      body: "Look 20 feet away for 20 seconds.",
      cue: "Blink slowly and let your eyes soften while it runs",
      action: "Done",
    },
  },
  {
    key: "posture",
    label: "Posture check",
    description: "A quick nudge to notice your position and shift it.",
    toast: {
      body: "Notice your position and shift it.",
      // ANSI/HFES 100 puts the monitor slightly below eye level, not at it.
      cue: "Ears over shoulders, shoulders relaxed, screen just below eye level",
      action: "Posture reset",
    },
  },
  {
    key: "movement",
    label: "Movement break",
    description: "Stand up and move for a minute or two.",
    toast: {
      body: "Stand up and move for a minute or two.",
      cue: "90 seconds on your feet — walk, roll your shoulders, stretch",
      action: "I'm up",
    },
  },
  {
    key: "hydration",
    label: "Hydration",
    description: "A reminder to drink some water.",
    toast: {
      body: "Time to drink some water.",
      cue: "One glass, about 250 ml",
      action: "Done",
    },
  },
];

export const CATEGORY_BY_KEY = Object.fromEntries(
  CATEGORIES.map((category) => [category.key, category]),
);

const minuteOptions = (minutesList) =>
  minutesList.map((minutes) => ({ value: minutes, label: `${minutes} min` }));

// Must match INTERVAL_OPTIONS_MINUTES / IDLE_OPTIONS_MINUTES in src-tauri/src/settings/model.rs.
export const INTERVAL_OPTIONS = minuteOptions([15, 20, 30, 45, 60]);
export const IDLE_OPTIONS = minuteOptions([3, 5, 10, 15, 30]);

function formatTwelveHour(hour, minute) {
  const period = hour < 12 ? "AM" : "PM";
  const displayHour = hour % 12 === 0 ? 12 : hour % 12;
  return `${displayHour}:${minute} ${period}`;
}

// Stored as "HH:MM" (24h), shown as "10:00 PM".
export const HALF_HOUR_TIME_OPTIONS = Array.from({ length: 48 }, (_, index) => {
  const hour = Math.floor(index / 2);
  const minute = index % 2 === 0 ? "00" : "30";
  return {
    value: `${String(hour).padStart(2, "0")}:${minute}`,
    label: formatTwelveHour(hour, minute),
  };
});
