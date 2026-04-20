<script lang="ts">
	type Props = {
		url?: string;
	};

	let { url }: Props = $props();
</script>

<div class="banner" style={url ? `--bg-img: url(${url})` : undefined}>
	<div class="overlay"></div>
</div>

<style>
	.banner {
		min-height: 140px;
		background: linear-gradient(120deg, #1e3a5f, #0f1419 55%, #152238);
		background-image: var(--bg-img);
		background-size: cover;
		background-position: center;
		position: relative;
	}

	.overlay {
		position: absolute;
		inset: 0;
		background: linear-gradient(to bottom, #0f141955, #0f1419cc);
		pointer-events: none;
	}
</style>
