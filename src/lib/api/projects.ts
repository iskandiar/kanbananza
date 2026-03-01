import { invoke } from '@tauri-apps/api/core';
import type { Project, Card } from '$lib/types';

export const listProjects = (archived = false): Promise<Project[]> =>
  invoke('list_projects', { archived });

export const createProject = (name: string, slug: string, color: string): Promise<Project> =>
  invoke('create_project', { name, slug, color });

export const updateProject = (
  id: number,
  fields: { name?: string; slug?: string; color?: string }
): Promise<Project> => invoke('update_project', { id, ...fields });

export const archiveProject = (id: number): Promise<void> =>
  invoke('archive_project', { id });

export const listCardsByProject = (projectId: number): Promise<Card[]> =>
  invoke('list_cards_by_project', { projectId });

export const generateProjectSlug = (name: string): Promise<string> =>
  invoke('generate_project_slug', { name });

export const summariseProject = (projectId: number): Promise<string> =>
  invoke('summarise_project', { projectId });
