import type { CachedSettings } from '$lib/types';

const KEY = 'court-admin:settings-cache';

export function loadCachedSettings(): CachedSettings | null {
	if (typeof localStorage === 'undefined') return null;
	const raw = localStorage.getItem(KEY);
	if (!raw) return null;
	try {
		const data = JSON.parse(raw) as CachedSettings;
		const maxAgeMs = data.retentionDays * 24 * 60 * 60 * 1000;
		if (Date.now() - data.savedAt > maxAgeMs) {
			localStorage.removeItem(KEY);
			return null;
		}
		return data;
	} catch {
		localStorage.removeItem(KEY);
		return null;
	}
}

export function saveCachedSettings(
	serviceAccountPath: string,
	retentionDays: number,
	bannerUrl?: string
): void {
	const payload: CachedSettings = {
		serviceAccountPath,
		retentionDays,
		bannerUrl,
		savedAt: Date.now()
	};
	localStorage.setItem(KEY, JSON.stringify(payload));
}

export function clearCachedSettings(): void {
	localStorage.removeItem(KEY);
}
