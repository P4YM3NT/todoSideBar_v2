import { ChevronLeft } from "lucide-react";
import { HANDLE_W } from "@/lib/ipc";
import { cn } from "@/lib/utils";

type Props = {
  expanded: boolean;
  onToggle: () => void;
};

/**
 * Der Griff bleibt beim Auf- und Zuklappen optisch an derselben Stelle stehen:
 * er ist in beiden Zustaenden vertikal zentriert und sitzt am linken Rand des
 * Fensters, das seinerseits rechtsbuendig am Bildschirm klebt.
 *
 * Bewusst kein `button` aus der Komponentenbibliothek: der Griff ist 14 px
 * breit und 100 px hoch: keine Groessenvariante trifft das, und alle bringen
 * Innenabstaende mit, die hier nicht passen.
 */
export function Handle({ expanded, onToggle }: Props) {
  return (
    <div className="flex h-full shrink-0 items-center" style={{ width: HANDLE_W }}>
      <button
        type="button"
        onClick={onToggle}
        aria-label={expanded ? "Sidebar einklappen" : "Sidebar ausklappen"}
        className="flex h-[100px] w-full items-center justify-center rounded-md
                   bg-card/80 text-muted-foreground backdrop-blur-sm outline-none
                   transition-colors hover:bg-accent hover:text-foreground
                   focus-visible:ring-3 focus-visible:ring-ring/50"
      >
        <ChevronLeft
          className={cn("size-3 transition-transform", expanded && "rotate-180")}
        />
      </button>
    </div>
  );
}
