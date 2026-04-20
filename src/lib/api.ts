import { invoke as tauriInvoke, isTauri } from '@tauri-apps/api/core';
import type { Reservation, ReservationWrite } from '$lib/types';

const NOT_IN_TAURI =
	'Database commands only work inside the desktop app. Stop using plain `npm run dev` for this screen and run `npm run tauri` instead (starts Vite + the Tauri shell with IPC).';

function assertTauriShell(): void {
	if (typeof window === 'undefined' || !isTauri()) {
		throw new Error(NOT_IN_TAURI);
	}
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	assertTauriShell();
	return tauriInvoke<T>(cmd, args ?? {});
}

export async function databaseConnect(serviceAccountPath: string): Promise<string> {
	return invoke<string>('database_connect', { serviceAccountPath });
}

export async function databaseDisconnect(): Promise<void> {
	return invoke('database_disconnect');
}

export async function databaseStatus(): Promise<boolean> {
	return invoke<boolean>('database_status');
}

export async function fetchReservationsWeek(weekStartIso: string): Promise<Reservation[]> {
	return invoke<Reservation[]>('fetch_reservations_week', { weekStartIso });
}

export async function reservationCreate(payload: ReservationWrite): Promise<string> {
	return invoke<string>('reservation_create', { payload });
}

export async function reservationUpdate(id: string, payload: ReservationWrite): Promise<void> {
	return invoke('reservation_update', { id, payload });
}

export async function reservationDelete(id: string): Promise<void> {
	return invoke('reservation_delete', { id });
}
