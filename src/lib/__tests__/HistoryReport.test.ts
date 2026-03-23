import { describe, it, expect } from 'vitest';

function buildPieInput(
  weekCards: { card_type: string; time_estimate: number | null }[],
  sessionTotalHours: number
): { card_type: string; hours: number }[] {
  const map: Record<string, number> = {};
  for (const c of weekCards) {
    map[c.card_type] = (map[c.card_type] ?? 0) + (c.time_estimate ?? 0);
  }
  const estimateTotal = Object.values(map).reduce((s, h) => s + h, 0);
  const other = Math.max(0, sessionTotalHours - estimateTotal);
  return [
    ...Object.entries(map).filter(([, h]) => h > 0).map(([card_type, hours]) => ({ card_type, hours })),
    ...(other > 0.01 ? [{ card_type: 'other', hours: other }] : []),
  ];
}

describe('buildPieInput', () => {
  it('builds slices from card estimates', () => {
    const cards = [
      { card_type: 'task', time_estimate: 3 },
      { card_type: 'task', time_estimate: 2 },
      { card_type: 'meeting', time_estimate: 1 },
    ];
    const result = buildPieInput(cards, 8);
    expect(result.find(s => s.card_type === 'task')?.hours).toBe(5);
    expect(result.find(s => s.card_type === 'meeting')?.hours).toBe(1);
    expect(result.find(s => s.card_type === 'other')?.hours).toBeCloseTo(2);
  });

  it('other is floored at 0 when estimates exceed clock-in', () => {
    const cards = [{ card_type: 'task', time_estimate: 10 }];
    const result = buildPieInput(cards, 6);
    expect(result.find(s => s.card_type === 'other')).toBeUndefined();
  });

  it('returns empty when no cards and no sessions', () => {
    expect(buildPieInput([], 0)).toHaveLength(0);
  });
});
