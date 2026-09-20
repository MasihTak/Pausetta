import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import ReminderToast from "./components/ReminderToast.vue";
import "./styles/base.scss";

// Rust injects this global only into the reminder window; every other window is Settings.
const isReminderWindow = Boolean(window.__PAUSETTA_REMINDER_CATEGORY__);
document.documentElement.classList.toggle("is-reminder", isReminderWindow);

createApp(isReminderWindow ? ReminderToast : App)
  .use(createPinia())
  .mount("#app");
