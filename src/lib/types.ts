// src/lib/types.ts

export type CardType = 'meeting' | 'mr' | 'thread' | 'task' | 'review' | 'documentation';
export type CardStatus = 'planned' | 'done';
export type Impact = 'low' | 'mid' | 'high';
export type Source = 'manual' | 'calendar' | 'gitlab' | 'linear' | 'slack' | 'notion';
export type AiProvider = 'anthropic' | 'openai';
export type IntegrationId = 'calendar' | 'gitlab' | 'linear' | 'slack' | 'notion';

export interface Card {
  id: number;
  title: string;
  card_type: CardType;
  status: CardStatus;
  impact: Impact | null;
  time_estimate: number | null; // hours
  url: string | null;
  week_id: number | null; // null = global backlog
  day_of_week: number | null; // 1=Mon..5=Fri, null = backlog
  position: number;
  source: Source;
  external_id: string | null;
  notes: string | null;
  metadata: string | null; // JSON string
  created_at: string;
  updated_at: string;
  project_id: number | null;
  done_at: string | null;
}

export interface Project {
  id: number;
  name: string;
  slug: string;        // 2-3 letter AI-generated, e.g. "API", "FRN"
  color: string;       // hex color, e.g. "#6366f1"
  archived: boolean;
  created_at: string;
}

export interface Week {
  id: number;
  year: number;
  week_number: number;
  start_date: string; // ISO date, Monday
  summary: string | null;
}

export interface Settings {
  id: number;
  available_hours: number;
  ai_provider: AiProvider | null;
  auto_ai: boolean;
}

export interface Integration {
  id: IntegrationId;
  enabled: boolean;
  config: string | null; // JSON string
  last_synced_at: string | null;
}

// Card metadata shapes (parsed from Card.metadata JSON)
export interface MeetingMetadata {
  start_time: string; // ISO datetime
  end_time: string;   // ISO datetime
}

export interface MrMetadata {
  author: string;
  mr_iid: number;
}

export interface ThreadMetadata {
  channel: string;
}

export interface TimeEntry {
  id: number;
  date: string;
  start_time: string;
  end_time: string | null;
  notes: string | null;
  created_at: string;
}

export interface CardTimeEntry {
  id: number;
  card_id: number;
  date: string;
  start_time: string;
  end_time: string | null;
  created_at: string;
}

export interface CardTypeHours {
  card_type: string;
  hours: number;
}

export interface DayTypeHours {
  date: string;       // YYYY-MM-DD
  card_type: string;
  hours: number;
}
