// ISO week number for a given date
export function isoWeek(date: Date): { year: number; weekNumber: number; startDate: string } {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  d.setDate(d.getDate() + 4 - (d.getDay() || 7));
  const yearStart = new Date(d.getFullYear(), 0, 1);
  const weekNumber = Math.ceil(((d.getTime() - yearStart.getTime()) / 86400000 + 1) / 7);
  // start = Monday of that week — use local date parts, not toISOString() which is UTC
  const start = new Date(date);
  start.setDate(date.getDate() - ((date.getDay() + 6) % 7));
  const startDate = [
    start.getFullYear(),
    String(start.getMonth() + 1).padStart(2, '0'),
    String(start.getDate()).padStart(2, '0')
  ].join('-');
  return { year: d.getFullYear(), weekNumber, startDate };
}

export function formatDateRange(startDate: string): string {
  const monday = new Date(startDate + 'T00:00:00');
  const friday = new Date(monday);
  friday.setDate(monday.getDate() + 4);
  const opts: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' };
  return `${monday.toLocaleDateString('en-US', opts)} – ${friday.toLocaleDateString('en-US', { ...opts, year: 'numeric' })}`;
}

// Sum time estimates across a list of cards
export function sumHours(cards: any[]): number {
  return cards.reduce((sum, c) => sum + (c.time_estimate ?? 0), 0);
}

/** Converts decimal hours to H:MM format. E.g. 0.4167 → "0:25", 1.5 → "1:30" */
export function hoursToHHMM(h: number): string {
  const totalMinutes = Math.round(h * 60);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  return `${hours}:${minutes.toString().padStart(2, '0')}`;
}

/** Parses "H:MM" or decimal hours string to decimal hours. Returns null for empty/invalid. */
export function parseHoursInput(s: string): number | null {
  const trimmed = s.trim();
  if (!trimmed) return null;
  const colonMatch = trimmed.match(/^(\d+):(\d{2})$/);
  if (colonMatch) {
    const h = parseInt(colonMatch[1], 10);
    const m = parseInt(colonMatch[2], 10);
    if (m >= 60) return null;
    return h + m / 60;
  }
  const n = parseFloat(trimmed);
  return isNaN(n) || n < 0 ? null : n;
}
