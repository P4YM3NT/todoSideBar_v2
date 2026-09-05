/**
 * Icon-Buttons (Papierkorb, Detail, Thema, Einstellungen).
 *
 * Die Aufhellung ist im hellen Theme schwaecher: dort mischt sich
 * --muted-foreground in Weiss und wird schnell zu dunkel. Im dunklen Theme
 * braucht es mehr, um sich vom Untergrund abzuheben.
 *
 * `bg-muted` waere semantisch naheliegend, ist im dunklen Theme aber derselbe
 * Wert wie --accent - und die Zeile liegt beim Hover bereits darauf.
 */
export const ICON_ACTION =
  "text-muted-foreground hover:bg-muted-foreground/12 dark:hover:bg-muted-foreground/25 hover:text-(--checkbox-accent)";
