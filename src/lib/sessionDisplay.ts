import type { Session } from "./types";

export function sessionLabel(s: Session): string {
  if (s.is_bg_agent) {
    return s.bg_name ?? s.bg_intent ?? s.title ?? "(no name)";
  }
  return s.title ?? "(no prompt yet)";
}

export function sessionMatchesQuery(s: Session, q: string): boolean {
  if (!q) return true;
  const needle = q.toLowerCase();
  const haystack = [
    s.title ?? "",
    s.recent_excerpt ?? "",
    s.bg_name ?? "",
    s.bg_intent ?? "",
    s.bg_detail ?? "",
  ].join(" ").toLowerCase();
  return haystack.includes(needle);
}
