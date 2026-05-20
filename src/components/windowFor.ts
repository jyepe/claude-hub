const MODEL_WINDOWS: Record<string, number> = {
  "claude-opus-4-7": 200_000,
  "claude-opus-4-7[1m]": 1_000_000,
  "claude-sonnet-4-6": 200_000,
  "claude-sonnet-4-6[1m]": 1_000_000,
  "claude-haiku-4-5-20251001": 200_000,
};

export function windowFor(
  model: string | null,
  maxPromptTokens: number,
  projectUsed1m: boolean,
  liveWindow: number | null = null,
): number {
  if (liveWindow && liveWindow > 0) return liveWindow;
  if (model && model.includes("[1m]")) return 1_000_000;
  if (maxPromptTokens > 200_000) return 1_000_000;
  if (projectUsed1m && model && /^claude-(opus|sonnet)-/.test(model)) {
    return 1_000_000;
  }
  if (!model) return 200_000;
  return MODEL_WINDOWS[model] ?? 200_000;
}
