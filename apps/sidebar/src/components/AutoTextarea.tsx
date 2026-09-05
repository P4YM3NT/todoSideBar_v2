import { useEffect, useRef } from "react";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";

/**
 * Textarea, die mit dem Inhalt waechst.
 *
 * `field-sizing: content` waere die CSS-Loesung, ist in WebKit aber noch nicht
 * verlaesslich verfuegbar - deshalb die Hoehe von Hand. Erst auf `auto`
 * zuruecksetzen, sonst kennt `scrollHeight` nur die bisherige Hoehe und das
 * Feld schrumpft beim Loeschen nie wieder.
 */
export function AutoTextarea({
  value,
  className,
  ...props
}: React.ComponentProps<typeof Textarea>) {
  const ref = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
  }, [value]);

  return (
    <Textarea
      ref={ref}
      value={value}
      rows={1}
      // Kein Ziehgriff: die Hoehe ergibt sich aus dem Inhalt.
      className={cn("resize-none overflow-hidden", className)}
      {...props}
    />
  );
}
