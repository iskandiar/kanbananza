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

type PieSlice = { label: string; hours: number; pct: number; color: string; startAngle: number; endAngle: number };

function buildPieSlices(input: { card_type: string; hours: number }[]): PieSlice[] {
  const typeColors: Record<string, string> = {
    task: '#10b981',
    meeting: '#3b82f6',
    mr: '#a855f7',
    thread: '#eab308',
    review: '#64748b',
    documentation: '#64748b',
  };

  function typeColor(type: string): string {
    return typeColors[type] ?? '#6b7280';
  }

  const total = input.reduce((s, b) => s + b.hours, 0);
  if (total === 0) return [];
  const threshold = total * 0.08;
  const main = input.filter(b => b.hours >= threshold);
  const otherHours = input.filter(b => b.hours < threshold).reduce((s, b) => s + b.hours, 0);
  const items: { label: string; hours: number; color: string }[] = [
    ...main.map(b => ({ label: b.card_type, hours: b.hours, color: typeColor(b.card_type) })),
    ...(otherHours > 0 ? [{ label: 'other', hours: otherHours, color: '#6b7280' }] : []),
  ];
  let angle = -Math.PI / 2;
  return items.map(item => {
    const sweep = (item.hours / total) * 2 * Math.PI;
    const slice: PieSlice = {
      label: item.label,
      hours: item.hours,
      pct: Math.round((item.hours / total) * 100),
      color: item.color,
      startAngle: angle,
      endAngle: angle + sweep,
    };
    angle += sweep;
    return slice;
  });
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

describe('buildPieSlices', () => {
  it('returns empty array for empty input', () => {
    expect(buildPieSlices([])).toEqual([]);
  });

  it('returns empty array when total hours is 0', () => {
    const input = [{ card_type: 'task', hours: 0 }];
    expect(buildPieSlices(input)).toEqual([]);
  });

  it('single item gets 100% and full circle', () => {
    const input = [{ card_type: 'task', hours: 8 }];
    const slices = buildPieSlices(input);

    expect(slices).toHaveLength(1);
    expect(slices[0].label).toBe('task');
    expect(slices[0].hours).toBe(8);
    expect(slices[0].pct).toBe(100);
    expect(slices[0].startAngle).toBe(-Math.PI / 2);
    expect(slices[0].endAngle).toBeCloseTo(3 * Math.PI / 2);
  });

  it('two equal items each get 50% and 180 degree sweep', () => {
    const input = [
      { card_type: 'task', hours: 4 },
      { card_type: 'meeting', hours: 4 },
    ];
    const slices = buildPieSlices(input);

    expect(slices).toHaveLength(2);
    expect(slices[0].pct).toBe(50);
    expect(slices[1].pct).toBe(50);

    // Each should sweep 180 degrees (π radians)
    expect(slices[0].endAngle - slices[0].startAngle).toBeCloseTo(Math.PI);
    expect(slices[1].endAngle - slices[1].startAngle).toBeCloseTo(Math.PI);
  });

  it('items below 8% threshold get merged into other', () => {
    const input = [
      { card_type: 'task', hours: 9 },     // 90%
      { card_type: 'meeting', hours: 0.5 }, // 5% — below threshold
      { card_type: 'mr', hours: 0.5 },      // 5% — below threshold
    ];
    const slices = buildPieSlices(input);

    expect(slices).toHaveLength(2);
    expect(slices[0].label).toBe('task');
    expect(slices[0].pct).toBe(90);
    expect(slices[1].label).toBe('other');
    expect(slices[1].hours).toBeCloseTo(1); // 0.5 + 0.5
    expect(slices[1].pct).toBe(10);
  });

  it('percentages sum to 100 (may be off by 1 due to rounding)', () => {
    const input = [
      { card_type: 'task', hours: 3 },
      { card_type: 'meeting', hours: 2 },
      { card_type: 'mr', hours: 2 },
      { card_type: 'thread', hours: 1 },
    ];
    const slices = buildPieSlices(input);

    const totalPct = slices.reduce((sum, s) => sum + s.pct, 0);
    expect(totalPct).toBeGreaterThanOrEqual(99);
    expect(totalPct).toBeLessThanOrEqual(101);
  });

  it('angles sweep from start to end correctly', () => {
    const input = [
      { card_type: 'task', hours: 5 },
      { card_type: 'meeting', hours: 5 },
    ];
    const slices = buildPieSlices(input);

    // Total angle swept should be 2π
    const totalSweep = slices.reduce((sum, s) => sum + (s.endAngle - s.startAngle), 0);
    expect(totalSweep).toBeCloseTo(2 * Math.PI);
  });
});
