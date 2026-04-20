/** Mirrors Firestore documents in the `bookings` collection. */
export type Reservation = {
	id: string;
	bookedBy: string;
	court: string;
	createdAt?: string;
	/** Local date YYYY-MM-DD */
	date: string;
	start_hour: number;
	end_hour: number;
	messengerUserId: string;
	sport: string;
	unit: string;
};

export type ReservationWrite = Omit<Reservation, 'id' | 'createdAt'>;

export type CachedSettings = {
	serviceAccountPath: string;
	retentionDays: number;
	bannerUrl?: string;
	savedAt: number;
};
