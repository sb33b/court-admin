<script lang="ts">
	import * as api from '$lib/api';
	import { enqueue, queueLength } from '$lib/offline-queue';
	import { courtsForSport, findFirstConflict, SPORTS } from '$lib/booking-conflicts';
	import type { Reservation, ReservationWrite } from '$lib/types';
	import { appState } from '$lib/app-state.svelte';

	type Props = {
		selectedId: string | null;
		slotDraft: { date: string; start_hour: number; end_hour: number } | null;
		onNew?: () => void;
		onSaved?: () => void | Promise<void>;
	};

	let { selectedId, slotDraft, onNew, onSaved }: Props = $props();

	let bookedBy = $state('');
	let court = $state('');
	let sport = $state('');
	let unit = $state('');
	/** Shown read-only when editing; not editable. */
	let originalMessenger = $state('');
	let dateStr = $state('');
	let startHour = $state<number>(8);
	let endHour = $state<number>(9);

	let busy = $state(false);
	let message = $state<string | null>(null);
	/** Inline confirm — `window.confirm` is unreliable in Tauri’s WebView. */
	let deleteConfirmOpen = $state(false);

	const courtOptions = $derived(courtsForSport(sport));

	function parseHour(n: number): number {
		return Number.isFinite(n) ? Math.trunc(n) : 0;
	}

	function onSportChange() {
		const opts = courtsForSport(sport);
		if (sport === 'Table Tennis') {
			court = 'Table Tennis';
		} else if (opts.length && !opts.includes(court)) {
			court = opts[0] ?? '';
		}
	}

	function loadFromReservation(r: Reservation) {
		bookedBy = r.bookedBy;
		court = r.court;
		sport = r.sport;
		unit = r.unit;
		originalMessenger = r.messengerUserId;
		dateStr = r.date;
		startHour = parseHour(r.start_hour);
		endHour = parseHour(r.end_hour);
	}

	function loadFromSlot(d: { date: string; start_hour: number; end_hour: number }) {
		bookedBy = '';
		court = '';
		sport = '';
		unit = '';
		originalMessenger = '';
		dateStr = d.date;
		startHour = parseHour(d.start_hour);
		endHour = parseHour(d.end_hour);
	}

	function clearEmpty() {
		bookedBy = '';
		court = '';
		sport = '';
		unit = '';
		originalMessenger = '';
		dateStr = '';
		startHour = 8;
		endHour = 9;
	}

	function buildPayload(): ReservationWrite | null {
		if (!/^\d{4}-\d{2}-\d{2}$/.test(dateStr.trim())) {
			message = 'Date must be YYYY-MM-DD.';
			return null;
		}
		const sh = parseHour(Number(startHour));
		const eh = parseHour(Number(endHour));
		if (!(sh < eh)) {
			message = 'End hour must be after start hour.';
			return null;
		}
		if (!sport || !(SPORTS as readonly string[]).includes(sport)) {
			message = 'Please select a sport.';
			return null;
		}
		if (!court || !courtOptions.includes(court)) {
			message = 'Please select a valid court for this sport.';
			return null;
		}
		if (!bookedBy.trim() || !unit.trim()) {
			message = 'Booked by and unit are required.';
			return null;
		}
		const messengerUserId = selectedId ? originalMessenger : 'admin';
		return {
			bookedBy: bookedBy.trim(),
			court,
			date: dateStr.trim(),
			start_hour: sh,
			end_hour: eh,
			messengerUserId,
			sport,
			unit: unit.trim()
		};
	}

	$effect(() => {
		message = null;
		deleteConfirmOpen = false;
		const id = selectedId;
		const slot = slotDraft;
		if (id) {
			const r = appState.reservations.find((x) => x.id === id);
			if (r) loadFromReservation(r);
			return;
		}
		if (slot) {
			loadFromSlot(slot);
			return;
		}
		clearEmpty();
	});

	async function handleSubmit() {
		message = null;
		const payload = buildPayload();
		if (!payload) return;
		if (!appState.connected) {
			message = 'Connect to the database first.';
			return;
		}
		const sameDay = appState.reservations.filter((r) => r.date === payload.date);
		const hit = findFirstConflict(payload, sameDay, selectedId);
		if (hit) {
			message = `Scheduling conflict with an existing booking (${hit.sport} · ${hit.court}).`;
			return;
		}
		busy = true;
		try {
			if (selectedId) {
				await api.reservationUpdate(selectedId, payload);
				message = 'Reservation updated.';
			} else {
				await api.reservationCreate(payload);
				message = 'Reservation created.';
			}
			await Promise.resolve(onSaved?.());
		} catch (e) {
			const err = e instanceof Error ? e.message : String(e);
			if (typeof navigator !== 'undefined' && !navigator.onLine) {
				if (selectedId) enqueue({ kind: 'update', id: selectedId, payload });
				else enqueue({ kind: 'create', payload });
				appState.offlineQueued = queueLength();
				message = 'Offline — change queued; will sync when online.';
			} else {
				message = err;
			}
		} finally {
			busy = false;
		}
	}

	function requestDelete() {
		if (!selectedId) return;
		message = null;
		deleteConfirmOpen = true;
	}

	function cancelDelete() {
		deleteConfirmOpen = false;
	}

	async function confirmDelete() {
		if (!selectedId) return;
		message = null;
		if (!appState.connected) {
			message = 'Connect to the database first.';
			deleteConfirmOpen = false;
			return;
		}
		busy = true;
		try {
			await api.reservationDelete(selectedId);
			deleteConfirmOpen = false;
			message = 'Deleted.';
			await Promise.resolve(onSaved?.());
			onNew?.();
		} catch (e) {
			const err = e instanceof Error ? e.message : String(e);
			if (typeof navigator !== 'undefined' && !navigator.onLine) {
				enqueue({ kind: 'delete', id: selectedId });
				appState.offlineQueued = queueLength();
				deleteConfirmOpen = false;
				message = 'Offline — delete queued; will sync when online.';
				onNew?.();
			} else {
				message = err;
			}
		} finally {
			busy = false;
		}
	}

	function handleNew() {
		onNew?.();
	}
</script>

<div class="panel">
	<div class="head">
		<h2>{selectedId ? 'Edit reservation' : 'New reservation'}</h2>
		<button type="button" class="linkish" onclick={handleNew}>Clear</button>
	</div>

	<label>Booked by<input bind:value={bookedBy} /></label>

	<label>Sport
		<select bind:value={sport} onchange={onSportChange}>
			<option value="">Select sport…</option>
			{#each SPORTS as s}
				<option value={s}>{s}</option>
			{/each}
		</select>
	</label>

	{#if sport}
		<label>Court
			<select bind:value={court}>
				<option value="">Select court…</option>
				{#each courtOptions as c}
					<option value={c}>{c}</option>
				{/each}
			</select>
		</label>
	{/if}

	<label>Unit<input bind:value={unit} /></label>

	{#if selectedId}
		<div class="field-ro">
			<span class="lbl">Messenger user ID</span>
			<span class="ro">{originalMessenger || '—'}</span>
			<span class="hint">Read-only; not updated when saving.</span>
		</div>
	{:else}
		<p class="admin-hint">Messenger user ID is saved as <strong>admin</strong> for new bookings.</p>
	{/if}

	<label>Date (YYYY-MM-DD)<input bind:value={dateStr} placeholder="2026-04-19" /></label>

	<div class="row2">
		<label>Start hour (integer)<input type="number" bind:value={startHour} min={0} max={23} step={1} /></label>
		<label>End hour (integer)<input type="number" bind:value={endHour} min={0} max={24} step={1} /></label>
	</div>

	<div class="actions">
		<button type="button" class="primary" disabled={busy} onclick={handleSubmit}>
			{busy ? 'Saving…' : selectedId ? 'Save changes' : 'Create reservation'}
		</button>
		{#if selectedId && !deleteConfirmOpen}
			<button type="button" class="danger" disabled={busy} onclick={requestDelete}>Delete</button>
		{/if}
	</div>

	{#if selectedId && deleteConfirmOpen}
		<div class="delete-confirm" role="dialog" aria-labelledby="delete-confirm-label">
			<p id="delete-confirm-label">Delete this reservation? This cannot be undone.</p>
			<div class="delete-confirm-actions">
				<button type="button" class="ghost" disabled={busy} onclick={cancelDelete}>Cancel</button>
				<button type="button" class="danger" disabled={busy} onclick={confirmDelete}>
					{busy ? 'Deleting…' : 'Delete'}
				</button>
			</div>
		</div>
	{/if}

	{#if message}
		<p class="msg">{message}</p>
	{/if}
</div>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 300px;
		max-width: 440px;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 14px;
		background: var(--surface);
		height: fit-content;
	}

	.head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 4px;
	}

	h2 {
		margin: 0;
		font-size: 1.05rem;
	}

	.linkish {
		border: none;
		background: none;
		color: var(--accent);
		padding: 0;
		font-size: 0.9rem;
	}

	label {
		display: grid;
		gap: 4px;
		font-size: 0.78rem;
		color: var(--muted);
	}

	input,
	select {
		padding: 8px 10px;
		border-radius: 8px;
		border: 1px solid var(--border);
		background: #0c1016;
	}

	.field-ro {
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 0.78rem;
		color: var(--muted);
	}

	.field-ro .lbl {
		font-size: 0.78rem;
	}

	.ro {
		padding: 8px 10px;
		border-radius: 8px;
		border: 1px solid var(--border);
		background: #0a0e14;
		color: var(--text);
		font-size: 0.9rem;
	}

	.field-ro .hint {
		font-size: 0.72rem;
		color: var(--muted);
	}

	.admin-hint {
		margin: 0;
		font-size: 0.82rem;
		color: var(--muted);
	}

	.row2 {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}

	.actions {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin-top: 4px;
	}

	.primary {
		background: var(--accent);
		border: none;
		padding: 9px 14px;
		border-radius: 8px;
		font-weight: 600;
	}
	.primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.danger {
		background: transparent;
		border: 1px solid #7f1d1d;
		color: #fecaca;
		padding: 9px 14px;
		border-radius: 8px;
	}

	.delete-confirm {
		margin-top: 6px;
		padding: 10px 12px;
		border-radius: 8px;
		border: 1px solid #7f1d1d;
		background: #1c0a0a;
	}

	.delete-confirm p {
		margin: 0 0 10px;
		font-size: 0.88rem;
		color: #fecaca;
	}

	.delete-confirm-actions {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		justify-content: flex-end;
	}

	.ghost {
		background: transparent;
		border: 1px solid var(--border);
		color: var(--text);
		padding: 9px 14px;
		border-radius: 8px;
	}

	.msg {
		margin: 0;
		font-size: 0.88rem;
		color: var(--muted);
	}
</style>
