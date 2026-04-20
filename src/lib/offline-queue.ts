import type { ReservationWrite } from '$lib/types';
import * as api from '$lib/api';

const QUEUE_KEY = 'court-admin:offline-queue';

export type QueuedOp =
	| { clientId: string; kind: 'create'; payload: ReservationWrite }
	| { clientId: string; kind: 'update'; id: string; payload: ReservationWrite }
	| { clientId: string; kind: 'delete'; id: string };

function load(): QueuedOp[] {
	if (typeof localStorage === 'undefined') return [];
	try {
		const raw = localStorage.getItem(QUEUE_KEY);
		if (!raw) return [];
		return JSON.parse(raw) as QueuedOp[];
	} catch {
		return [];
	}
}

function save(ops: QueuedOp[]): void {
	localStorage.setItem(QUEUE_KEY, JSON.stringify(ops));
}

export function queueLength(): number {
	return load().length;
}

export function enqueue(
	op:
		| { kind: 'create'; payload: ReservationWrite; clientId?: string }
		| { kind: 'update'; id: string; payload: ReservationWrite; clientId?: string }
		| { kind: 'delete'; id: string; clientId?: string }
): void {
	const ops = load();
	const clientId = op.clientId ?? crypto.randomUUID();
	const next: QueuedOp =
		op.kind === 'create'
			? { kind: 'create', clientId, payload: op.payload }
			: op.kind === 'update'
				? { kind: 'update', clientId, id: op.id, payload: op.payload }
				: { kind: 'delete', clientId, id: op.id };
	ops.push(next);
	save(ops);
}

export function clearQueue(): void {
	localStorage.removeItem(QUEUE_KEY);
}

/** Process queued writes against the Rust backend. Stops on first failure. */
export async function flushQueue(
	onProgress?: (remaining: number) => void
): Promise<{ ok: boolean; error?: string }> {
	let ops = load();
	while (ops.length > 0) {
		const op = ops[0];
		try {
			if (op.kind === 'create') {
				await api.reservationCreate(op.payload);
			} else if (op.kind === 'update') {
				await api.reservationUpdate(op.id, op.payload);
			} else {
				await api.reservationDelete(op.id);
			}
			ops = ops.slice(1);
			save(ops);
			onProgress?.(ops.length);
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			if (typeof navigator !== 'undefined' && !navigator.onLine) {
				return { ok: false, error: 'offline' };
			}
			return { ok: false, error: msg };
		}
	}
	return { ok: true };
}
