import { describe, it, expect, beforeEach, vi } from 'vitest';
import { projectsStore } from '../stores/projects.svelte';
import * as projectsApi from '../api/projects';
import * as cardsApi from '../api/cards';
import type { Card } from '$lib/types';

vi.mock('$lib/api/projects');
vi.mock('$lib/api/cards');
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}));

const createMockCard = (overrides: Partial<Card> = {}): Card => ({
  id: 1,
  title: 'Test',
  card_type: 'task',
  status: 'planned',
  impact: null,
  time_estimate: null,
  url: null,
  week_id: null,
  day_of_week: null,
  position: 0,
  source: 'manual',
  external_id: null,
  notes: null,
  metadata: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  project_id: 1,
  done_at: null,
  ...overrides
});

beforeEach(() => {
  vi.clearAllMocks();
  projectsStore.projectCardsMap = new Map();
});

describe('backlogFor / inProgressFor / doneFor', () => {
  it('backlogFor returns only week_id===null && status===planned for that project', () => {
    const card1 = createMockCard({ id: 1, project_id: 1, week_id: null, status: 'planned' });
    const card2 = createMockCard({ id: 2, project_id: 1, week_id: 5, status: 'planned' });
    const card3 = createMockCard({ id: 3, project_id: 1, week_id: null, status: 'done' });
    const card4 = createMockCard({ id: 4, project_id: 2, week_id: null, status: 'planned' });
    projectsStore.projectCardsMap = new Map([
      [1, [card1, card2, card3]],
      [2, [card4]]
    ]);

    expect(projectsStore.backlogFor(1)).toEqual([card1]);
  });

  it('inProgressFor returns only week_id!==null && status===planned for that project', () => {
    const card1 = createMockCard({ id: 1, project_id: 1, week_id: null, status: 'planned' });
    const card2 = createMockCard({ id: 2, project_id: 1, week_id: 5, status: 'planned' });
    const card3 = createMockCard({ id: 3, project_id: 1, week_id: null, status: 'done' });
    projectsStore.projectCardsMap = new Map([[1, [card1, card2, card3]]]);

    expect(projectsStore.inProgressFor(1)).toEqual([card2]);
  });

  it('doneFor returns only status===done for that project', () => {
    const card1 = createMockCard({ id: 1, project_id: 1, week_id: null, status: 'planned' });
    const card2 = createMockCard({ id: 2, project_id: 1, week_id: 5, status: 'planned' });
    const card3 = createMockCard({ id: 3, project_id: 1, week_id: null, status: 'done' });
    projectsStore.projectCardsMap = new Map([[1, [card1, card2, card3]]]);

    expect(projectsStore.doneFor(1)).toEqual([card3]);
  });

  it('backlogFor for project 2 does not bleed cards from project 1', () => {
    const card1 = createMockCard({ id: 1, project_id: 1, week_id: null, status: 'planned' });
    const card2 = createMockCard({ id: 2, project_id: 1, week_id: 5, status: 'planned' });
    const card3 = createMockCard({ id: 3, project_id: 1, week_id: null, status: 'done' });
    const card4 = createMockCard({ id: 4, project_id: 2, week_id: null, status: 'planned' });
    projectsStore.projectCardsMap = new Map([
      [1, [card1, card2, card3]],
      [2, [card4]]
    ]);

    const result = projectsStore.backlogFor(2);
    expect(result).toEqual([card4]);
    expect(result).not.toContainEqual(expect.objectContaining({ id: 1 }));
  });

  it('returns empty array for unknown project ID', () => {
    expect(projectsStore.backlogFor(999)).toEqual([]);
    expect(projectsStore.inProgressFor(999)).toEqual([]);
    expect(projectsStore.doneFor(999)).toEqual([]);
  });
});

describe('loadProjectCards', () => {
  it('calls listCardsByProject and stores under projectCardsMap', async () => {
    const cards = [createMockCard({ id: 1, project_id: 1 })];
    vi.spyOn(projectsApi, 'listCardsByProject').mockResolvedValue(cards);

    await projectsStore.loadProjectCards(1);

    expect(projectsApi.listCardsByProject).toHaveBeenCalledWith(1);
    expect(projectsStore.projectCardsMap.get(1)).toEqual(cards);
  });

  it('does not call API again when cards already cached', async () => {
    const cards = [createMockCard({ id: 1, project_id: 1 })];
    projectsStore.projectCardsMap = new Map([[1, cards]]);
    const spy = vi.spyOn(projectsApi, 'listCardsByProject');

    await projectsStore.loadProjectCards(1);

    expect(spy).not.toHaveBeenCalled();
  });
});

describe('markDone', () => {
  it('moves card from inProgressFor to doneFor after call', async () => {
    const inProgressCard = createMockCard({
      id: 10,
      project_id: 1,
      week_id: 5,
      status: 'planned'
    });
    const doneCard = createMockCard({ id: 10, project_id: 1, week_id: 5, status: 'done' });
    projectsStore.projectCardsMap = new Map([[1, [inProgressCard]]]);

    vi.spyOn(cardsApi, 'updateCard').mockResolvedValue(doneCard);

    await projectsStore.markDone(10);

    expect(projectsStore.doneFor(1)).toContainEqual(doneCard);
    expect(projectsStore.inProgressFor(1)).not.toContainEqual(
      expect.objectContaining({ id: 10 })
    );
  });
});

describe('moveToBacklog', () => {
  it('moves card from inProgressFor to backlogFor after call', async () => {
    const inProgressCard = createMockCard({
      id: 20,
      project_id: 1,
      week_id: 5,
      status: 'planned'
    });
    const backlogCard = createMockCard({
      id: 20,
      project_id: 1,
      week_id: null,
      status: 'planned'
    });
    projectsStore.projectCardsMap = new Map([[1, [inProgressCard]]]);

    vi.spyOn(cardsApi, 'updateCard').mockResolvedValue(backlogCard);

    await projectsStore.moveToBacklog(20);

    expect(projectsStore.backlogFor(1)).toContainEqual(backlogCard);
    expect(projectsStore.inProgressFor(1)).not.toContainEqual(
      expect.objectContaining({ id: 20 })
    );
  });
});

describe('addCardToProject', () => {
  it('new card appears in backlogFor after call', async () => {
    const newCard = createMockCard({
      id: 99,
      project_id: 1,
      week_id: null,
      status: 'planned'
    });
    projectsStore.projectCardsMap = new Map([[1, []]]);

    vi.spyOn(cardsApi, 'createCard').mockResolvedValue(newCard);

    await projectsStore.addCardToProject('New task', 1);

    expect(projectsStore.backlogFor(1)).toContainEqual(newCard);
  });
});
