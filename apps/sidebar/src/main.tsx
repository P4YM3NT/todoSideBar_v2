import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { App } from "./App";
import { TaskDetail } from "@/components/TaskDetail";
import { TooltipProvider } from "@/components/ui/tooltip";
import { applyMirroredTheme } from "@/lib/theme";
import "./styles.css";

applyMirroredTheme();

// Beide Fenster laden dasselbe Bundle; das Label entscheidet, was gerendert
// wird. Ein zweiter HTML-Einstiegspunkt waere ein zweiter Build fuer denselben
// Code.
const Root = getCurrentWindow().label === "detail" ? TaskDetail : App;

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <TooltipProvider>
      <Root />
    </TooltipProvider>
  </React.StrictMode>,
);
