import { useEffect, useState } from "react";
import { api } from "../lib/api";

interface Props {
  onChange: () => void;
}

export function HiddenProjectsManager({ onChange }: Props) {
  const [hidden, setHidden] = useState<string[]>([]);
  const [open, setOpen] = useState(false);

  const reload = async () => {
    const prefs = await api.getPrefs();
    setHidden(prefs.hidden_projects);
  };

  useEffect(() => {
    if (open) reload();
  }, [open]);

  const unhide = async (path: string) => {
    await api.unhideProject(path);
    await reload();
    onChange();
  };

  return (
    <div className="relative">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="px-3 py-1 text-sm rounded-md border border-border bg-surface hover:bg-surface-hi text-text-2"
      >
        Hidden ({hidden.length})
      </button>
      {open && (
        <div className="absolute right-0 mt-2 w-[480px] z-10 bg-surface border border-border rounded-md p-3 shadow-lg">
          {hidden.length === 0 && (
            <div className="text-text-3 text-sm">No projects hidden.</div>
          )}
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
  );
}
