import App from "./App.svelte";
import "./style.css";

const container = document.getElementById("app");
if (!container) {
  throw new Error("App container not found");
}

const app = new App({
  target: container,
});

export default app;
