import { useEffect, useState } from "react";
import {
  collapseSidebar,
  getSidebarState,
  onClosed,
  onCloseRequested,
  onOpen,
} from "./ipc";

/**
 * `opening` und `closing` existieren, weil Rahmenwechsel und Animation
 * getrennt laufen: beim Öffnen ist das Fenster schon groß, während der Inhalt
 * noch rechts außerhalb liegt; beim Schließen muss die Animation fertig sein,
 * bevor der Rahmen schrumpfen darf.
 */
export type Phase = "collapsed" | "opening" | "expanded" | "closing";

export function useSidebarPhase() {
  const [phase, setPhase] = useState<Phase>("collapsed");

  useEffect(() => {
    let alive = true;
    getSidebarState().then((s) => {
      if (alive) setPhase(s.expanded ? "expanded" : "collapsed");
    });

    const subs = [
      onOpen(() => setPhase("opening")),
      onCloseRequested(() => setPhase("closing")),
      onClosed(() => setPhase("collapsed")),
    ];

    return () => {
      alive = false;
      subs.forEach((p) => p.then((un) => un()));
    };
  }, []);

  // Zwei Frames warten, damit der Startzustand garantiert gemalt wurde -
  // sonst überspringt der Browser die Transition.
  useEffect(() => {
    if (phase !== "opening") return;
    let inner = 0;
    const outer = requestAnimationFrame(() => {
      inner = requestAnimationFrame(() => setPhase("expanded"));
    });
    return () => {
      cancelAnimationFrame(outer);
      cancelAnimationFrame(inner);
    };
  }, [phase]);

  const onTransitionEnd = (e: React.TransitionEvent) => {
    // Nur der Slide des Containers zählt, nicht Transitions von Kindelementen.
    if (e.propertyName !== "transform" || e.currentTarget !== e.target) return;
    if (phase === "closing") void collapseSidebar();
  };

  return { phase, onTransitionEnd };
}
