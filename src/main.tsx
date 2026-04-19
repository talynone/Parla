import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { MiniRecorderView } from "./components/MiniRecorderView";
import "./i18n";
import "./index.css";

// Selection de la vue en fonction du label de fenetre :
// - "main" : app principale
// - "recorder" : mini-recorder flottant
const label = getCurrentWindow().label;
const isRecorder = label === "recorder";
const Root = isRecorder ? MiniRecorderView : App;

// La fenetre mini-recorder est transparente au niveau Tauri (decorations off,
// transparent: true). On force le body / #root en transparent pour eviter
// que le bg blanc tailwind par defaut colle un carre blanc derriere la pill.
if (isRecorder) {
  document.documentElement.style.background = "transparent";
  document.body.style.background = "transparent";
  document.body.style.colorScheme = "dark";
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
