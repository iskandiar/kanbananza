// src/lib/types.ts

export type CardType = 'meeting' | 'mr' | 'thread' | 'task' | 'review';
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
