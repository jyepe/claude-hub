export interface Session {
  id: string;
  jsonl_path: string;
  cwd: string | null;
  title: string | null;
  model: string | null;
  message_count: number;
  tokens: number;
  context_tokens: number;
  max_prompt_tokens: number;
  last_activity: string | null;
  live_context_window: number | null;
  live_model_id: string | null;
  is_bg_agent: boolean;
}

export interface Worktree {
  path: string;
  sessions: Session[];
}

export interface Project {
  path: string;
  display_name: string;
  session_count: number;
  total_tokens: number;
  last_activity: string | null;
  sessions: Session[];
  worktrees: Worktree[];
  hidden: boolean;
  used_1m_recently: boolean;
}

export interface Stats {
  project_count: number;
  session_count: number;
  tokens_7d: number;
  tokens_all_time: number;
}

export interface Prefs {
  hidden_projects: string[];
  noise_threshold: number;
}
