import { useEffect, useState } from "react";
import { api } from "../lib/api";

interface Props {
  count: number;
  open: boolean;
  onClose: () => void;
  onChange: () => void;
}

export function HiddenProjectsManager({ count, open, onClose, onChange }: Props) {
  const [hidden, setHidden] = useState<string[]>([]);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    (async () => {
      const prefs = await api.getPrefs();
      if (!cancelled) setHidden(prefs.hidden_projects);
    })();
    return () => {
      cancelled = true;
    };
  }, [open]);

  if (!open) return null;

  const unhide = async (path: string) => {
    await api.unhideProject(path);
    const prefs = await api.getPrefs();
    setHidden(prefs.hidden_projects);
    onChange();
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Hidden projects"
      className="fixed inset-0 z-30 flex items-start justify-center pt-24 bg-black/40"
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-[480px] bg-surface border border-border rounded-md p-4 shadow-[0_1px_2px_rgba(0,0,0,0.12),0_8px_24px_rgba(0,0,0,0.28)]"
      >
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-text-1 text-sm font-semibold">
            Hidden projects ({count})
          </h2>
          <button
            type="button"
            aria-label="Close"
            onClick={onClose}
            className="text-text-3 hover:text-text-2 transition-colors duration-[120ms]"
          >
            ×
          </button>
        </div>
        {hidden.length === 0 ? (
          <div className="text-text-3 text-sm">No projects hidden.</div>
        ) : (
          <div className="flex flex-col">
            {hidden.map((p) => (
              <div
                key={p}
                className="flex items-center justify-between gap-3 py-1"
              >
                <span className="font-mono text-xs text-text-2 truncate">{p}</span>
                <button
                  type="button"
                  onClick={() => unhide(p)}
                  className="text-xs text-accent hover:text-accent-hover"
                >
                  unhide
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
