<script lang="ts">
	import { onMount } from 'svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import CoverBanner from '$lib/components/CoverBanner.svelte';
	import WeekCalendar from '$lib/components/WeekCalendar.svelte';
	import ReservationPanel from '$lib/components/ReservationPanel.svelte';
	import { loadCachedSettings } from '$lib/session-cache';
	import * as api from '$lib/api';
	import { flushQueue, queueLength } from '$lib/offline-queue';
	import { appState } from '$lib/app-state.svelte';
	import { startOfWeekMonday, addDays, toIsoDate } from '$lib/time';

	let bannerUrl = $state(loadCachedSettings()?.bannerUrl);

	let selectedId = $state<string | null>(null);
	let slotDraft = $state<{ date: string; start_hour: number; end_hour: number } | null>(null);

	async function refreshWeek() {
		if (!appState.connected) return;
		const monday = addDays(startOfWeekMonday(new Date()), appState.weekOffset * 7);
		const weekStartIso = toIsoDate(monday);
		appState.reservationsLoading = true;
		try {
			appState.reservations = await api.fetchReservationsWeek(weekStartIso);
			appState.connectionMessage = null;
		} catch (e) {
			appState.connectionMessage = e instanceof Error ? e.message : String(e);
			appState.reservations = [];
		} finally {
			appState.reservationsLoading = false;
		}
	}

	function syncOfflineAndRefresh() {
		appState.offlineQueued = queueLength();
		return flushQueue((n) => {
			appState.offlineQueued = n;
		}).then((r) => {
			if (!r.ok && r.error && r.error !== 'offline') {
				appState.connectionMessage = r.error;
			}
			return refreshWeek();
		});
	}

	onMount(() => {
		const s = loadCachedSettings();
		const run = async () => {
			if (!s?.serviceAccountPath) return;
			try {
				await api.databaseConnect(s.serviceAccountPath);
				appState.connected = true;
				await syncOfflineAndRefresh();
			} catch (e) {
				appState.connected = false;
				appState.connectionMessage = e instanceof Error ? e.message : String(e);
			}
		};
		void run();

		const onOnline = () => {
			void syncOfflineAndRefresh();
		};
		window.addEventListener('online', onOnline);
		return () => window.removeEventListener('online', onOnline);
	});

	$effect(() => {
		const connected = appState.connected;
		appState.weekOffset;
		if (!connected) return;
		void refreshWeek();
	});

	function onPickReservation(id: string) {
		selectedId = id;
		slotDraft = null;
	}

	function onPickSlot(slot: { date: string; start_hour: number; end_hour: number }) {
		selectedId = null;
		slotDraft = slot;
	}

	function clearSelection() {
		selectedId = null;
		slotDraft = null;
	}

	function refreshBanner() {
		bannerUrl = loadCachedSettings()?.bannerUrl;
	}
</script>

<div class="shell">
	<TopBar
		connected={appState.connected}
		errorText={appState.connectionMessage}
		onConnected={async () => {
			refreshBanner();
			await syncOfflineAndRefresh();
		}}
		onDisconnected={() => {
			refreshBanner();
			clearSelection();
			appState.reservations = [];
			appState.reservationsLoading = false;
		}}
	/>

	<CoverBanner url={bannerUrl} />

	{#if appState.offlineQueued > 0}
		<div class="queue-banner">
			{appState.offlineQueued} operation(s) waiting in offline queue — will sync when online.
		</div>
	{/if}

	<main class="main">
		<div class="left">
			<WeekCalendar onSelectReservation={onPickReservation} onSelectSlot={onPickSlot} />
		</div>
		<aside class="aside">
			<ReservationPanel
				{selectedId}
				{slotDraft}
				onNew={clearSelection}
				onSaved={refreshWeek}
			/>
		</aside>
	</main>
</div>

<style>
	.shell {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}

	.queue-banner {
		padding: 8px 14px;
		background: #422006;
		border-bottom: 1px solid #854d0e;
		font-size: 0.88rem;
		color: #fef3c7;
	}

	.main {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: 14px;
		padding: 14px;
		flex: 1;
		min-height: 0;
		align-items: start;
	}

	.left {
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.aside {
		position: sticky;
		top: 12px;
	}
</style>
