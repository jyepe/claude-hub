import { useEffect } from "react";

interface Props {
  project: { path: string; name: string };
  onUndo: () => void;
  onDismiss: () => void;
}

export function UndoToast({ project, onUndo, onDismiss }: Props) {
  useEffect(() => {
    const t = setTimeout(onDismiss, 5000);
    return () => clearTimeout(t);
  }, [project.path, onDismiss]);

  return (
    <div
      role="status"
      aria-live="polite"
      className="fixed bottom-4 right-4 z-20 inline-flex items-center gap-3 bg-surface-hi border border-border rounded-md px-3 py-2 text-sm text-text-1 shadow-[0_1px_2px_rgba(0,0,0,0.12),0_8px_24px_rgba(0,0,0,0.28)]"
    >
      <span>
        Hidden <strong className="font-semibold">{project.name}</strong>
      </span>
      <button
        type="button"
        onClick={onUndo}
        className="text-accent hover:text-accent-hover font-semibold transition-colors duration-[120ms]"
      >
        Undo
      </button>
      <button
        type="button"
        aria-label="Dismiss"
        onClick={onDismiss}
        className="text-text-3 hover:text-text-2 transition-colors duration-[120ms]"
      >
        ×
      </button>
    </div>
  );
}
