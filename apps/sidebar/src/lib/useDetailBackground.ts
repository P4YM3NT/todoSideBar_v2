import { useEffect } from "react";

import { resolveRgb } from "@/lib/accent";
import { setDetailBackground } from "@/lib/detail";

/**
 * Meldet Rust die Flächenfarbe des gerade geltenden Themes.
 *
 * Rust braucht sie an zwei Stellen: beim Vergrößern des Detailfensters, sonst
 * blitzt die graue Standardfläche auf, bevor der WebView den neuen Bereich
 * gezeichnet hat - und beim *Erzeugen* des Fensters, das sonst immer weiß
 * entsteht und im dunklen Theme sichtbar aufblitzt.
 *
 * Deshalb hängt der Haken in beiden Fenstern: die Sidebar hält den Wert aktuell,
 * auch wenn noch nie ein Detailfenster offen war.
 *
 * Der Beobachter fängt "system" mit ab - dort wechselt die Klasse ohne
 * Zutun des Nutzers.
 */
export function useDetailBackground() {
  useEffect(() => {
    const sync = () => {
      const [r, g, b] = resolveRgb("var(--background)");
      void setDetailBackground(r, g, b);
    };
    sync();

    const observer = new MutationObserver(sync);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });
    return () => observer.disconnect();
  }, []);
}
