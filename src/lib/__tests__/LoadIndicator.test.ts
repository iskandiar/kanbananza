import { describe, it, expect } from 'vitest';

/**
 * Extract the load indicator calculation logic for testing.
 * This mirrors the derived values in LoadIndicator.svelte
 */
function calculateLoadIndicator(
  doneHours: number,
  plannedHours: number,
  availableHours: number
): { donePct: number; plannedPct: number; total: number } {
  const total = doneHours + plannedHours;
  const donePct = availableHours > 0 ? Math.min((doneHours / availableHours) * 100, 100) : 0;
  const plannedPct = availableHours > 0 ? Math.min((plannedHours / availableHours) * 100, Math.max(0, 100 - donePct)) : 0;

  return { donePct, plannedPct, total };
}

describe('LoadIndicator', () => {
  describe('availableHours = 0', () => {
    it('returns 0 for both percentages (no divide-by-zero)', () => {
      const result = calculateLoadIndicator(0, 0, 0);
      expect(result.donePct).toBe(0);
      expect(result.plannedPct).toBe(0);
      expect(result.total).toBe(0);
    });

    it('returns 0 for both percentages even with nonzero hours', () => {
      const result = calculateLoadIndicator(5, 3, 0);
      expect(result.donePct).toBe(0);
      expect(result.plannedPct).toBe(0);
      expect(result.total).toBe(8);
    });
  });

  describe('doneHours > availableHours', () => {
    it('clamps done to 100%, planned to 0%', () => {
      const result = calculateLoadIndicator(10, 5, 8);
      expect(result.donePct).toBe(100);
      expect(result.plannedPct).toBe(0);
      expect(result.total).toBe(15);
    });

    it('clamps even when planned hours are positive', () => {
      const result = calculateLoadIndicator(12, 10, 15);
      expect(result.donePct).toBe(80);
      expect(result.plannedPct).toBe(Math.min((10 / 15) * 100, Math.max(0, 100 - 80)));
      expect(result.plannedPct).toBe(20);
    });
  });

  describe('doneHours + plannedHours > availableHours', () => {
    it('caps planned to remaining capacity after done', () => {
      const result = calculateLoadIndicator(3, 5, 6);
      const donePct = (3 / 6) * 100; // 50%
      expect(result.donePct).toBe(donePct);
      // plannedPct should be capped to 100 - 50 = 50%
      expect(result.plannedPct).toBe(50);
      expect(result.total).toBe(8);
    });

    it('handles fractional hours', () => {
      const result = calculateLoadIndicator(2.5, 3.5, 5);
      const donePct = (2.5 / 5) * 100; // 50%
      expect(result.donePct).toBe(donePct);
      // plannedPct should be (3.5 / 5) * 100 = 70%, capped to 100 - 50 = 50%
      expect(result.plannedPct).toBe(50);
      expect(result.total).toBe(6);
    });
  });

  describe('normal capacity scenarios', () => {
    it('returns correct percentages when within capacity', () => {
      const result = calculateLoadIndicator(2, 3, 8);
      expect(result.donePct).toBe((2 / 8) * 100); // 25%
      expect(result.plannedPct).toBe((3 / 8) * 100); // 37.5%
      expect(result.total).toBe(5);
    });

    it('returns correct percentages at exactly full capacity', () => {
      const result = calculateLoadIndicator(4, 4, 8);
      expect(result.donePct).toBe(50);
      expect(result.plannedPct).toBe(50);
      expect(result.total).toBe(8);
    });

    it('handles zero done hours', () => {
      const result = calculateLoadIndicator(0, 5, 10);
      expect(result.donePct).toBe(0);
      expect(result.plannedPct).toBe(50);
      expect(result.total).toBe(5);
    });

    it('handles zero planned hours', () => {
      const result = calculateLoadIndicator(5, 0, 10);
      expect(result.donePct).toBe(50);
      expect(result.plannedPct).toBe(0);
      expect(result.total).toBe(5);
    });
  });

  describe('edge cases', () => {
    it('handles very small available hours', () => {
      const result = calculateLoadIndicator(0.1, 0.1, 0.3);
      expect(result.donePct).toBeCloseTo((0.1 / 0.3) * 100, 5);
      expect(result.plannedPct).toBeCloseTo((0.1 / 0.3) * 100, 5);
    });

    it('handles very large numbers', () => {
      const result = calculateLoadIndicator(1000, 500, 2000);
      expect(result.donePct).toBe(50);
      expect(result.plannedPct).toBe(25);
      expect(result.total).toBe(1500);
    });
  });
});
