<script lang="ts">
	import {
		startOfWeekMonday,
		addDays,
		formatDayTitle,
		formatWeekRange,
		toIsoDate,
		hourRows,
		formatHourLabel
	} from '$lib/time';
	import type { Reservation } from '$lib/types';
	import { appState } from '$lib/app-state.svelte';

	type Props = {
		onSelectReservation: (id: string) => void;
		onSelectSlot: (slot: { date: string; start_hour: number; end_hour: number }) => void;
	};

	let { onSelectReservation, onSelectSlot }: Props = $props();

	const weekStart = $derived(addDays(startOfWeekMonday(new Date()), appState.weekOffset * 7));
	const weekLabel = $derived(formatWeekRange(weekStart));

	const hours = $derived(hourRows());
	const dayDates = $derived(Array.from({ length: 7 }, (_, i) => addDays(weekStart, i)));

	/** Half-open overlap: [start_hour, end_hour) vs row hour. */
	function reservationsForCell(isoDate: string, hour: number): Reservation[] {
		return appState.reservations.filter((r) => {
			if (r.date !== isoDate) return false;
			return r.start_hour <= hour && hour < r.end_hour;
		});
	}

	/** Left-click: select an existing booking in the cell, or start a new one if empty. */
	function handleCellClick(isoDate: string, hour: number, e: MouseEvent) {
		const t = e.target as HTMLElement;
		if (t.closest('button.res')) return;
		const list = reservationsForCell(isoDate, hour);
		if (list.length > 0) {
			onSelectReservation(list[0].id);
			return;
		}
		onSelectSlot({ date: isoDate, start_hour: hour, end_hour: hour + 1 });
	}

	/** Right-click: always treat as empty cell — start a new booking for this slot. */
	function handleCellContextMenu(isoDate: string, hour: number, e: MouseEvent) {
		e.preventDefault();
		onSelectSlot({ date: isoDate, start_hour: hour, end_hour: hour + 1 });
	}
</script>

<section class="cal-wrap">
	<div class="cal-head">
		<div class="nav">
			<button type="button" class="nav-btn" onclick={() => appState.weekOffset--}>← Prev</button>
			<div class="title">{weekLabel}</div>
			<button type="button" class="nav-btn" onclick={() => appState.weekOffset++}>Next →</button>
			<button type="button" class="today" onclick={() => (appState.weekOffset = 0)}>This week</button>
		</div>
		{#if appState.connected}
			<p class="week-status" aria-live="polite">
				{#if appState.reservationsLoading}
					Loading reservations from Firestore…
				{:else}
					{appState.reservations.length} reservation{appState.reservations.length === 1 ? '' : 's'} in this week’s range
				{/if}
			</p>
		{/if}
	</div>

	<div class="table-scroll">
		<table class="grid">
			<thead>
				<tr>
					<th class="corner">Hour</th>
					{#each dayDates as d}
						<th>{formatDayTitle(d)}</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each hours as hour}
					<tr>
						<th class="time">{formatHourLabel(hour)}</th>
						{#each dayDates as d}
							{@const iso = toIsoDate(d)}
							<td
								class="cell"
								onclick={(e) => handleCellClick(iso, hour, e)}
								oncontextmenu={(e) => handleCellContextMenu(iso, hour, e)}
							>
								<div class="cell-inner">
									{#each reservationsForCell(iso, hour) as r}
										<button
											type="button"
											class="res"
											onclick={() => onSelectReservation(r.id)}
											title={r.bookedBy}
										>
											<span class="tag">{r.sport}</span>
											<span class="main">{r.court}</span>
											<span class="sub">{r.bookedBy}</span>
										</button>
									{/each}
								</div>
							</td>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</section>

<style>
	.cal-wrap {
		display: flex;
		flex-direction: column;
		min-height: 0;
		flex: 1;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		overflow: hidden;
		background: var(--surface);
	}

	.cal-head {
		padding: 10px 12px;
		border-bottom: 1px solid var(--border);
		background: var(--surface-2);
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.week-status {
		margin: 0;
		font-size: 0.82rem;
		color: var(--muted);
	}

	.nav {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.title {
		font-weight: 650;
		flex: 1;
		min-width: 160px;
	}

	.nav-btn,
	.today {
		border: 1px solid var(--border);
		background: #0c1016;
		padding: 6px 10px;
		border-radius: 8px;
	}

	.today {
		margin-left: auto;
	}

	.table-scroll {
		overflow: auto;
		max-height: min(62vh, 720px);
	}

	.grid {
		width: 100%;
		border-collapse: separate;
		border-spacing: 0;
		font-size: 0.82rem;
	}

	th,
	td {
		border-bottom: 1px solid var(--border);
		border-right: 1px solid var(--border);
		vertical-align: top;
	}

	th {
		background: #121923;
		position: sticky;
		top: 0;
		z-index: 2;
		padding: 8px;
	}

	.corner {
		left: 0;
		z-index: 3;
		min-width: 72px;
	}

	.time {
		position: sticky;
		left: 0;
		z-index: 1;
		background: #121923;
		font-weight: 500;
		color: var(--muted);
		padding: 8px;
		width: 72px;
	}

	.cell {
		min-width: 120px;
		max-width: 180px;
		padding: 4px;
		cursor: cell;
		background: #0c1016;
	}

	.cell-inner {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-height: 28px;
	}

	.res {
		text-align: left;
		border: 1px solid var(--border);
		background: var(--accent-dim);
		border-radius: 8px;
		padding: 4px 6px;
		display: grid;
		gap: 2px;
	}

	.res:hover {
		border-color: var(--accent);
	}

	.tag {
		font-size: 0.7rem;
		color: #bfdbfe;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.main {
		font-weight: 600;
	}

	.sub {
		font-size: 0.75rem;
		color: var(--muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
