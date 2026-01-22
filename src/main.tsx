import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./style.css";

const container = document.getElementById("app");
if (!container) {
  throw new Error("App container not found");
}

ReactDOM.createRoot(container).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
