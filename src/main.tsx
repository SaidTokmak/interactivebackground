import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// Kalıcı tercih SQLite'tan yüklenene kadar işletim sistemi temasını kullanarak
// ilk frame'de açık tema parlamasını önleriz.
const initialTheme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
document.documentElement.dataset.theme = initialTheme;
document.documentElement.style.colorScheme = initialTheme;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
