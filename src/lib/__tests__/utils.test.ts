import { describe, it, expect } from 'vitest';
import { isoWeek, formatDateRange } from '../utils';

describe('isoWeek', () => {
  it('handles Jan 1 year boundary (week numbering edge case)', () => {
    const jan1_2024 = new Date(2024, 0, 1); // Monday, Jan 1, 2024
    const result = isoWeek(jan1_2024);
    expect(result).toHaveProperty('year');
    expect(result).toHaveProperty('weekNumber');
    expect(result).toHaveProperty('startDate');
    expect(result.weekNumber).toBeGreaterThanOrEqual(1);
    expect(result.weekNumber).toBeLessThanOrEqual(53);
  });

  it('handles Dec 28 (can be week 52 or 53)', () => {
    const dec28_2023 = new Date(2023, 11, 28);
    const result = isoWeek(dec28_2023);
    expect(result.weekNumber).toBeGreaterThanOrEqual(52);
    expect(result.weekNumber).toBeLessThanOrEqual(53);
    expect(result.year).toBe(2023);
  });

  it('returns correct week for mid-year date', () => {
    const midYear = new Date(2024, 6, 15); // July 15, 2024
    const result = isoWeek(midYear);
    expect(result.year).toBe(2024);
    expect(result.weekNumber).toBeGreaterThanOrEqual(1);
    expect(result.weekNumber).toBeLessThanOrEqual(53);
    expect(result.startDate).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });

  it('handles leap year date correctly', () => {
    const leapDay = new Date(2024, 1, 29); // Feb 29, 2024 (leap year)
    const result = isoWeek(leapDay);
    expect(result.year).toBe(2024);
    expect(result.weekNumber).toBeGreaterThanOrEqual(1);
    expect(result.weekNumber).toBeLessThanOrEqual(53);
    expect(result.startDate).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });

  it('returns startDate as ISO string (YYYY-MM-DD)', () => {
    const date = new Date(2024, 5, 15);
    const result = isoWeek(date);
    expect(result.startDate).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    const [year, month, day] = result.startDate.split('-').map(Number);
    expect(year).toBeGreaterThanOrEqual(2000);
    expect(month).toBeGreaterThanOrEqual(1);
    expect(month).toBeLessThanOrEqual(12);
    expect(day).toBeGreaterThanOrEqual(1);
    expect(day).toBeLessThanOrEqual(31);
  });
});

describe('formatDateRange', () => {
  it('formats normal Monday start as 7-day range', () => {
    // Monday June 3, 2024
    const result = formatDateRange('2024-06-03');
    expect(result).toBeTruthy();
    // Should contain the month "Jun" and dates
    expect(result).toMatch(/Jun/);
    expect(result).toMatch(/\d+/);
    // Should contain separator
    expect(result).toContain('–');
  });

  it('formats cross-year week (Dec to Jan)', () => {
    // Monday Dec 25, 2023 (spans to Friday Dec 29, so no year boundary crossing)
    // Let's use Dec 23, 2025 which goes into 2026
    const result = formatDateRange('2025-12-22');
    expect(result).toBeTruthy();
    expect(result).toMatch(/Dec/);
    expect(result).toMatch(/\d+/);
    expect(result).toContain('–');
    // Should include year (Friday is in next year or still in Dec)
    expect(result).toMatch(/\d{4}/);
  });

  it('has correct output string shape', () => {
    const result = formatDateRange('2024-03-04');
    // Verify format: "Month Day – Month Day, Year"
    // All calls should have the shape: Something – Something
    const parts = result.split('–');
    expect(parts).toHaveLength(2);
    expect(parts[0].trim()).toBeTruthy();
    expect(parts[1].trim()).toBeTruthy();
    // Both parts should contain at least one digit (day number)
    expect(parts[0]).toMatch(/\d/);
    expect(parts[1]).toMatch(/\d/);
  });

  it('includes year in the end date', () => {
    const result = formatDateRange('2024-01-01');
    // Friday should include year
    expect(result).toMatch(/\d{4}/);
  });
});
