/**
 * Start hours (24h) for each calendar row. Each row is [start, start+1), e.g.
 * 6 → 6–7am, 18 → 6–7pm, … 21 → 9–10pm.
 */
export const CALENDAR_ROW_START_HOURS = [6, 18, 19, 20, 21] as const;

/** Monday 00:00:00 local time for the week containing `date`. */
export function startOfWeekMonday(date: Date): Date {
	const d = new Date(date);
	const day = d.getDay();
	const diff = day === 0 ? -6 : 1 - day;
	d.setDate(d.getDate() + diff);
	d.setHours(0, 0, 0, 0);
	return d;
}

export function addDays(date: Date, days: number): Date {
	const d = new Date(date);
	d.setDate(d.getDate() + days);
	return d;
}

export function formatDayTitle(d: Date): string {
	return d.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' });
}

export function formatWeekRange(weekStart: Date): string {
	const weekEnd = addDays(weekStart, 6);
	return `${weekStart.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })} – ${weekEnd.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}`;
}

/** Local calendar date as YYYY-MM-DD */
export function toIsoDate(d: Date): string {
	const y = d.getFullYear();
	const m = String(d.getMonth() + 1).padStart(2, '0');
	const day = String(d.getDate()).padStart(2, '0');
	return `${y}-${m}-${day}`;
}

/** Row start hours shown in the grid (order preserved). */
export function hourRows(): number[] {
	return [...CALENDAR_ROW_START_HOURS];
}

/** e.g. "6am", "10pm" (no ":00" for whole hours). */
function formatCompactHour(hour24: number): string {
	const d = new Date();
	d.setHours(hour24, 0, 0, 0);
	const s = d.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
	return s.replace(':00', '').replace(/\s+/g, '').toLowerCase();
}

/** Label for one row: start-end of that hour slot (e.g. `9pm-10pm`). */
export function formatHourLabel(startHour: number): string {
	return `${formatCompactHour(startHour)}-${formatCompactHour(startHour + 1)}`;
}
