import type { Project, Card } from '$lib/types';
import * as projectsApi from '$lib/api/projects';
import * as cardsApi from '$lib/api/cards';

class ProjectsStore {
  projects: Project[] = $state([]);
  projectCardsMap: Map<number, Card[]> = $state(new Map());
  isLoading: boolean = $state(false);

  backlogFor(projectId: number): Card[] {
    return (this.projectCardsMap.get(projectId) ?? []).filter(
      (c) => c.week_id === null && c.status === 'planned'
    );
  }

  inProgressFor(projectId: number): Card[] {
    return (this.projectCardsMap.get(projectId) ?? []).filter(
      (c) => c.week_id !== null && c.status === 'planned'
    );
  }

  doneFor(projectId: number): Card[] {
    return (this.projectCardsMap.get(projectId) ?? []).filter((c) => c.status === 'done');
  }

  cardsLoadedFor(projectId: number): boolean {
    return this.projectCardsMap.has(projectId);
  }

  async loadProjects(): Promise<void> {
    this.isLoading = true;
    try {
      this.projects = await projectsApi.listProjects(false);
    } finally {
      this.isLoading = false;
    }
  }

  async loadProjectCards(projectId: number): Promise<void> {
    if (this.projectCardsMap.has(projectId)) return;
    const cards = await projectsApi.listCardsByProject(projectId);
    this.projectCardsMap = new Map(this.projectCardsMap).set(projectId, cards);
  }

  async createProject(name: string, color: string): Promise<Project> {
    let slug: string;
    try {
      slug = await projectsApi.generateProjectSlug(name);
    } catch {
      slug = name.slice(0, 3).toUpperCase();
    }
    const project = await projectsApi.createProject(name, slug, color);
    await this.loadProjects();
    return project;
  }

  async archiveProject(id: number): Promise<void> {
    await projectsApi.archiveProject(id);
    const next = new Map(this.projectCardsMap);
    next.delete(id);
    this.projectCardsMap = next;
    await this.loadProjects();
  }

  async moveToInProgress(cardId: number, currentWeekId: number, dayOfWeek: number): Promise<void> {
    const card = await cardsApi.updateCard(cardId, { weekId: currentWeekId, dayOfWeek });
    this._patchCard(cardId, card);
  }

  async moveToBacklog(cardId: number): Promise<void> {
    const card = await cardsApi.updateCard(cardId, { clearWeek: true });
    this._patchCard(cardId, card);
  }

  async markDone(cardId: number): Promise<void> {
    const card = await cardsApi.updateCard(cardId, { status: 'done' });
    this._patchCard(cardId, card);
  }

  async moveFromDoneToBacklog(cardId: number): Promise<void> {
    const card = await cardsApi.updateCard(cardId, { status: 'planned', clearWeek: true });
    this._patchCard(cardId, card);
  }

  async addCardToProject(title: string, projectId: number): Promise<void> {
    const card = await cardsApi.createCard(title, 'task', null, null, projectId);
    const existing = this.projectCardsMap.get(projectId) ?? [];
    this.projectCardsMap = new Map(this.projectCardsMap).set(projectId, [...existing, card]);
  }

  async deleteCard(cardId: number): Promise<void> {
    await cardsApi.deleteCard(cardId);
    this._removeCard(cardId);
  }

  // Sync an externally-updated card into the map (e.g. after EditCardModal saves).
  syncCard(updatedCard: Card): void {
    this._patchCard(updatedCard.id, updatedCard);
  }

  private _patchCard(cardId: number, updatedCard: Card): void {
    for (const [pid, cards] of this.projectCardsMap) {
      const idx = cards.findIndex((c) => c.id === cardId);
      if (idx >= 0) {
        const next = [...cards];
        next[idx] = updatedCard;
        this.projectCardsMap = new Map(this.projectCardsMap).set(pid, next);
        return;
      }
    }
  }

  private _removeCard(cardId: number): void {
    for (const [pid, cards] of this.projectCardsMap) {
      if (cards.some((c) => c.id === cardId)) {
        this.projectCardsMap = new Map(this.projectCardsMap).set(
          pid,
          cards.filter((c) => c.id !== cardId)
        );
        return;
      }
    }
  }
}

export const projectsStore = new ProjectsStore();
