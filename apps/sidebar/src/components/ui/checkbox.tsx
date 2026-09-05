import { Checkbox as CheckboxPrimitive } from "@base-ui/react/checkbox"

import { cn } from "@/lib/utils"
import { CheckIcon } from "lucide-react"

// Zwei Abweichungen vom shadcn-Original, beide wegen derselben Ursache:
// `dark:`-Klassen erzeugen `:is(.dark *)` und haben damit zwei Klassen
// Spezifitaet, `data-checked:` erzeugt `:where(...)` und hat nur eine. Jede
// `dark:`-Regel schlaegt also jede `data-checked:`-Regel, egal in welcher
// Reihenfolge sie stehen - und egal, was `cn()` am Aufrufort setzt.
//
//   1. `dark:bg-input/30` ist auf `dark:not-data-checked:bg-input/30`
//      eingeschraenkt. Es ist die Flaeche der *leeren* Box; unbeschraenkt
//      uebertoente es in Dunkel auch die gefuellte und liess nur den farbigen
//      Rahmen stehen.
//   2. `dark:data-checked:bg-primary` ist entfernt. Es sagt dasselbe wie
//      `data-checked:bg-primary`, ueberlebt als eigene Variantengruppe aber
//      jedes `cn()` und faerbte die abgehakte Box in Dunkel weiss statt in der
//      gewaehlten Akzentfarbe - weisser Haken auf weisser Flaeche.
//
// Nach `shadcn add checkbox` beides wieder herstellen.
function Checkbox({ className, ...props }: CheckboxPrimitive.Root.Props) {
  return (
    <CheckboxPrimitive.Root
      data-slot="checkbox"
      className={cn(
        "peer relative flex size-4 shrink-0 items-center justify-center rounded-[4px] border border-input transition-colors outline-none group-has-disabled/field:opacity-50 group-has-[:focus-visible]/field-label:ring-0 group-has-[:focus-visible]/field-label:not-data-checked:border-input after:absolute after:-inset-x-3 after:-inset-y-2 focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 aria-invalid:aria-checked:border-primary dark:not-data-checked:bg-input/30 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 data-checked:border-primary data-checked:bg-primary data-checked:text-primary-foreground group-has-[:focus-visible]/field-label:data-checked:border-primary",
        className
      )}
      {...props}
    >
      <CheckboxPrimitive.Indicator
        data-slot="checkbox-indicator"
        className="grid place-content-center text-current transition-none [&>svg]:size-3.5"
      >
        <CheckIcon
        />
      </CheckboxPrimitive.Indicator>
    </CheckboxPrimitive.Root>
  )
}

export { Checkbox }
