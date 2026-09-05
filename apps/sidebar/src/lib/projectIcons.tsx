import {
  Briefcase,
  Code,
  Dumbbell,
  Folder,
  GraduationCap,
  Heart,
  Home,
  Palette,
  Plane,
  ShoppingCart,
  Users,
  Wrench,
} from "lucide-react";

/**
 * Feste Auswahl statt aller Lucide-Symbole: nur was hier steht, landet im
 * Bundle. Der gespeicherte Wert ist der Schluessel - unbekannte Namen fallen
 * auf `Folder` zurueck, damit ein spaeter entferntes Symbol nichts zerbricht.
 */
export const PROJECT_ICONS = {
  folder: Folder,
  briefcase: Briefcase,
  home: Home,
  study: GraduationCap,
  heart: Heart,
  travel: Plane,
  shopping: ShoppingCart,
  code: Code,
  sport: Dumbbell,
  design: Palette,
  tools: Wrench,
  people: Users,
} as const;

export type ProjectIconName = keyof typeof PROJECT_ICONS;

export const projectIcon = (name: string | null) =>
  PROJECT_ICONS[(name ?? "folder") as ProjectIconName] ?? Folder;
