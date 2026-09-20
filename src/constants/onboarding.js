// One slide per reminder category, after a welcome. Bodies and notes follow
// science-research.md: state what the evidence supports, and say so when it doesn't.
export const ONBOARDING_SLIDES = [
  {
    key: "welcome",
    title: "Welcome to Pausetta",
    body: "A quiet companion for anyone who sits at a screen all day. Every so often it nudges you to look after your eyes, posture, movement, and hydration, then gets out of the way.",
  },
  {
    key: "eye",
    title: "Eye care",
    body: "Every 20 minutes, look at something about 20 feet away for 20 seconds. Screen work cuts how often you blink, which dries your eyes out and tires the muscles that focus them.",
    note: "The exact numbers aren't settled by research, but the habit costs nothing and is worth keeping.",
  },
  {
    key: "posture",
    title: "Posture check",
    body: "The longer you hold one position, the more it loads your neck and shoulders. This is a nudge to notice where you're sitting and reset it: ears over shoulders, screen just below eye level. That works better than trying to hold one perfect posture all day.",
    note: "No study pins down the right interval for this. It works best alongside movement breaks.",
  },
  {
    key: "movement",
    title: "Movement break",
    body: "Standing up every 30 to 60 minutes, even for a minute or two, is linked to lower cardiometabolic risk and less musculoskeletal discomfort. Studies find no measurable cost to how much you get done.",
    note: "The best supported habit in this app, across cohort studies and workplace trials alike.",
  },
  {
    key: "hydration",
    title: "Hydration",
    body: "It's easy to drift into mild dehydration during a long stretch of focused work. It measurably affects mood and concentration, even when it doesn't show up in tests of task accuracy.",
    note: "A steady glass of water is well justified by the evidence, without overselling it.",
  },
];
