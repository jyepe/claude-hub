import { invoke } from "@tauri-apps/api/core";
import type { Project, Stats, Prefs } from "./types";

export const api = {
  listProjects: () => invoke<Project[]>("list_projects"),
  getStats: () => invoke<Stats>("get_stats"),
  getPrefs: () => invoke<Prefs>("get_prefs"),
  setPrefs: (prefs: Prefs) => invoke<void>("set_prefs", { new: prefs }),
  hideProject: (path: string) => invoke<void>("hide_project", { path }),
  unhideProject: (path: string) => invoke<void>("unhide_project", { path }),
  openSession: (cwd: string, resumeId?: string) =>
    invoke<void>("open_session", { cwd, resumeId: resumeId ?? null }),
  attachAgent: (cwd: string, sessionId: string) =>
    invoke<void>("attach_agent", { cwd, sessionId }),
};
