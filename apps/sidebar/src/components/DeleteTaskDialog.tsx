import { useEffect, useState } from "react";
import { type Task, listSubtasks } from "@/lib/tasks";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

type Props = {
  /** Die Aufgabe, um die es geht. `null` heißt: nichts zu bestätigen. */
  task: Task | null;
  onOpenChange: (open: boolean) => void;
  onConfirm: (task: Task) => void;
};

/**
 * Rückfrage vor dem Löschen. Löschen ist die einzige Aktion in der App, die
 * sich nicht rückgängig machen lässt - abgehakte Aufgaben kann man wieder
 * öffnen, verschobene zurückschieben.
 *
 * Die Zahl der Unteraufgaben wird erst beim Öffnen geholt: sie ist genau dann
 * interessant, wenn man löschen will, und ein Zähler an jeder Listenzeile wäre
 * eine Abfrage pro Zeile.
 */
export function DeleteTaskDialog({ task, onOpenChange, onConfirm }: Props) {
  const [subtasks, setSubtasks] = useState(0);

  useEffect(() => {
    if (!task) return;
    let alive = true;
    setSubtasks(0);
    void listSubtasks(task.id).then((list) => {
      if (alive) setSubtasks(list.length);
    });
    return () => {
      alive = false;
    };
  }, [task]);

  return (
    <AlertDialog open={task !== null} onOpenChange={onOpenChange}>
      <AlertDialogContent size="sm">
        <AlertDialogHeader>
          <AlertDialogTitle>Aufgabe löschen?</AlertDialogTitle>
          <AlertDialogDescription>
            „{task?.title}“ wird gelöscht.
            {subtasks > 0 &&
              ` Die ${subtasks === 1 ? "Unteraufgabe" : `${subtasks} Unteraufgaben`} ${
                subtasks === 1 ? "verschwindet" : "verschwinden"
              } mit.`}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel size="sm">Abbrechen</AlertDialogCancel>
          {/* Kein `Close`, deshalb schließt der Knopf selbst - sonst bliebe der
              Dialog nach dem Löschen über einer Zeile stehen, die es nicht mehr gibt. */}
          <AlertDialogAction
            size="sm"
            variant="destructive"
            onClick={() => {
              if (!task) return;
              onOpenChange(false);
              onConfirm(task);
            }}
          >
            Löschen
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
