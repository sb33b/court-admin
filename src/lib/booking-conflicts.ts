import type { Reservation, ReservationWrite } from '$lib/types';

export const SPORTS = ['Basketball', 'Pickleball', 'Table Tennis'] as const;
export type Sport = (typeof SPORTS)[number];

export function courtsForSport(sport: string): string[] {
	switch (sport) {
		case 'Basketball':
			return ['1', '2', 'Full'];
		case 'Pickleball':
			return ['A', 'B', 'C'];
		case 'Table Tennis':
			return ['Table Tennis'];
		default:
			return [];
	}
}

export function timeRangesOverlap(s1: number, e1: number, s2: number, e2: number): boolean {
	return s1 < e2 && s2 < e1;
}

/** True if proposed conflicts with existing (same rules as server). */
export function schedulingConflict(
	proposed: ReservationWrite,
	existing: Reservation,
	excludeExistingId?: string | null
): boolean {
	if (excludeExistingId && existing.id === excludeExistingId) return false;
	if (proposed.date !== existing.date) return false;
	if (!timeRangesOverlap(proposed.start_hour, proposed.end_hour, existing.start_hour, existing.end_hour)) {
		return false;
	}
	if (proposed.court === existing.court) return true;
	if (basketballFullVsHalves(proposed, existing)) return true;
	const pFull = proposed.sport === 'Basketball' && proposed.court === 'Full';
	const eFull = existing.sport === 'Basketball' && existing.court === 'Full';
	const pPb = proposed.sport === 'Pickleball' && ['A', 'B', 'C'].includes(proposed.court);
	const ePb = existing.sport === 'Pickleball' && ['A', 'B', 'C'].includes(existing.court);
	const fullVsPb = (pFull && ePb) || (eFull && pPb);
	if (fullVsPb) return true;
	return bbCourtVsPickleballAdjacent(proposed, existing);
}

/** Basketball Full vs courts 1 or 2 (same floor). */
function basketballFullVsHalves(p: ReservationWrite, e: Reservation): boolean {
	const pFull = p.sport === 'Basketball' && p.court === 'Full';
	const eFull = e.sport === 'Basketball' && e.court === 'Full';
	const pHalf = p.sport === 'Basketball' && (p.court === '1' || p.court === '2');
	const eHalf = e.sport === 'Basketball' && (e.court === '1' || e.court === '2');
	return (pFull && eHalf) || (eFull && pHalf);
}

/** Pickleball C ↔ Basketball 1; Pickleball B ↔ Basketball 2. */
function bbCourtVsPickleballAdjacent(p: ReservationWrite, e: Reservation): boolean {
	const pBb1 = p.sport === 'Basketball' && p.court === '1';
	const eBb1 = e.sport === 'Basketball' && e.court === '1';
	const pPbc = p.sport === 'Pickleball' && p.court === 'C';
	const ePbc = e.sport === 'Pickleball' && e.court === 'C';
	const pBb2 = p.sport === 'Basketball' && p.court === '2';
	const eBb2 = e.sport === 'Basketball' && e.court === '2';
	const pPbb = p.sport === 'Pickleball' && p.court === 'B';
	const ePbb = e.sport === 'Pickleball' && e.court === 'B';
	return (pBb1 && ePbc) || (eBb1 && pPbc) || (pBb2 && ePbb) || (eBb2 && pPbb);
}

export function findFirstConflict(
	proposed: ReservationWrite,
	existingList: Reservation[],
	excludeId?: string | null
): Reservation | undefined {
	return existingList.find((ex) => schedulingConflict(proposed, ex, excludeId ?? undefined));
}
