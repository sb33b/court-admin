<script lang="ts">
	import { saveCachedSettings, clearCachedSettings, loadCachedSettings } from '$lib/session-cache';
	import * as api from '$lib/api';
	import { appState } from '$lib/app-state.svelte';

	type Props = {
		connected: boolean;
		errorText: string | null;
		onConnected: () => void | Promise<void>;
		onDisconnected: () => void;
	};

	let { connected, errorText, onConnected, onDisconnected }: Props = $props();

	let expanded = $state(false);

	const cached = loadCachedSettings();

	let serviceAccountPath = $state(cached?.serviceAccountPath ?? '');
	let retentionDays = $state(cached?.retentionDays ?? 14);
	let bannerUrl = $state(cached?.bannerUrl ?? '');

	let busy = $state(false);
	let localError = $state<string | null>(null);

	async function handleConnect() {
		localError = null;
		busy = true;
		try {
			await api.databaseConnect(serviceAccountPath.trim());
			saveCachedSettings(serviceAccountPath.trim(), retentionDays, bannerUrl.trim() || undefined);
			appState.connected = true;
			appState.connectionMessage = null;
			expanded = false;
			await Promise.resolve(onConnected());
		} catch (e) {
			localError = e instanceof Error ? e.message : 'Connection failed';
			appState.connected = false;
		} finally {
			busy = false;
		}
	}

	async function handleDisconnect() {
		localError = null;
		busy = true;
		try {
			await api.databaseDisconnect();
			clearCachedSettings();
			appState.connected = false;
			appState.reservations = [];
			onDisconnected();
		} catch (e) {
			localError = e instanceof Error ? e.message : 'Disconnect failed';
		} finally {
			busy = false;
		}
	}
</script>

<header class="topbar">
	<div class="brand">Buena Vida Multisport Court - Administrator Dashboard</div>

	<div class="right">
		{#if !expanded}
			<button
				type="button"
				class="pill"
				onclick={() => (expanded = true)}
				title="Service account connection"
			>
				<span class="dot" class:ok={connected} class:bad={!connected}></span>
				<span class="pill-text">Database</span>
			</button>
		{:else}
			<div class="panel" role="dialog" aria-label="Firestore service account">
				<div class="panel-head">
					<strong>Service account</strong>
					<button type="button" class="close" onclick={() => (expanded = false)}>×</button>
				</div>

				<p class="hint">
					Path to your Google Cloud service account JSON (Firestore access). Credentials never leave
					this machine except as signed API calls to Google.
				</p>

				<label
					>JSON key file path<input
						bind:value={serviceAccountPath}
						placeholder="/path/to/service-account.json"
						autocomplete="off"
					/></label
				>

				<label>Remember path (days)<input type="number" min="1" max="365" bind:value={retentionDays} /></label>
				<label
					>Cover image URL (optional)<input
						bind:value={bannerUrl}
						placeholder="https://…"
						autocomplete="off"
					/></label
				>

				<div class="actions">
					<button type="button" class="primary" disabled={busy} onclick={handleConnect}>
						{busy ? 'Connecting…' : 'Connect & save'}
					</button>
					<button type="button" class="ghost" disabled={busy || !connected} onclick={handleDisconnect}>
						Disconnect
					</button>
				</div>

				{#if localError}
					<p class="err">{localError}</p>
				{/if}
				{#if errorText}
					<p class="err soft">{errorText}</p>
				{/if}
			</div>
		{/if}
	</div>
</header>

<style>
	.topbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 14px;
		border-bottom: 1px solid var(--border);
		background: var(--surface);
		gap: 12px;
	}

	.brand {
		font-weight: 650;
		letter-spacing: 0.02em;
	}

	.right {
		position: relative;
	}

	.pill {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		border-radius: 999px;
		padding: 6px 12px;
	}

	.pill-text {
		font-size: 0.9rem;
	}

	.dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--muted);
		box-shadow: 0 0 0 2px #0003 inset;
	}
	.dot.ok {
		background: var(--ok);
	}
	.dot.bad {
		background: var(--danger);
	}

	.panel {
		position: absolute;
		right: 0;
		top: calc(100% + 8px);
		width: min(420px, 92vw);
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 12px;
		display: grid;
		gap: 8px;
		z-index: 20;
		box-shadow: 0 12px 40px #0008;
	}

	.panel-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 4px;
	}

	.hint {
		margin: 0;
		font-size: 0.78rem;
		color: var(--muted);
		line-height: 1.35;
	}

	.close {
		border: none;
		background: transparent;
		font-size: 1.25rem;
		line-height: 1;
		padding: 4px 8px;
		color: var(--muted);
	}

	label {
		display: grid;
		gap: 4px;
		font-size: 0.78rem;
		color: var(--muted);
	}

	input {
		padding: 8px 10px;
		border-radius: 8px;
		border: 1px solid var(--border);
		background: #0c1016;
	}

	.actions {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.primary {
		background: var(--accent);
		border: none;
		padding: 8px 12px;
		border-radius: 8px;
		font-weight: 600;
	}
	.primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.ghost {
		background: transparent;
		border: 1px solid var(--border);
		padding: 8px 12px;
		border-radius: 8px;
	}

	.err {
		color: #fecaca;
		font-size: 0.85rem;
		margin: 0;
	}
	.err.soft {
		color: #fde68a;
	}
</style>
