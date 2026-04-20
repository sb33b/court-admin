import type { Reservation } from '$lib/types';

export const appState = $state({
	connected: false,
	connectionMessage: null as string | null,
	/** Reservations for the visible week (from Firestore). */
	reservations: [] as Reservation[],
	/** True while fetching the week from the backend. */
	reservationsLoading: false,
	weekOffset: 0,
	offlineQueued: 0
});
