import { useEffect, useRef } from "react";
import { setSidebarWidth } from "@/lib/ipc";

/**
 * Verbreitert die Zone zum Ziehen der linken Kante.
 *
 * macOS behandelt die aeussersten ~4 px des Fensters selbst und laesst sie gar
 * nicht erst zum WebView durch - eine schmalere Flaeche als diese bekaeme
 * niemals ein Ereignis. Dieser Streifen ist breiter, sodass sein innerer Teil
 * uns erreicht. Aussen zieht weiterhin macOS, innen wir; beides endet in
 * derselben Fenstergroesse.
 *
 * Gespeichert wird hier bewusst nicht: der `onResized`-Horcher in `App` sieht
 * jede Groessenaenderung ohnehin - egal ob sie von macOS oder von hier kommt -
 * und schreibt sie einmal fest, wenn das Ziehen zur Ruhe kommt.
 */
const MIN_WIDTH = 220;

export function ResizeGrip() {
  const drag = useRef<{ startX: number; startWidth: number } | null>(null);
  const frame = useRef<number | null>(null);

  useEffect(() => {
    // Am Fenster, nicht am Streifen: der Zeiger verlaesst ihn beim Ziehen sofort.
    const onMove = (e: PointerEvent) => {
      const d = drag.current;
      if (!d) return;
      // Linke Kante nach links ziehen macht die Sidebar breiter.
      const next = Math.max(MIN_WIDTH, d.startWidth + (d.startX - e.screenX));
      if (frame.current !== null) cancelAnimationFrame(frame.current);
      frame.current = requestAnimationFrame(() => void setSidebarWidth(next));
    };

    const onUp = () => {
      drag.current = null;
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onUp);
      if (frame.current !== null) cancelAnimationFrame(frame.current);
    };
  }, []);

  return (
    <div
      onPointerDown={(e) => {
        if (e.button !== 0) return;
        e.preventDefault();
        // `window.innerWidth` ist synchron und immer die echte Fensterbreite.
        drag.current = { startX: e.screenX, startWidth: window.innerWidth };
      }}
      aria-hidden
      className="absolute left-0 top-0 z-10 h-full w-3 cursor-col-resize"
    />
  );
}
