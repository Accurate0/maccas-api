<script lang="ts">
	import Points from '$lib/components/tabs/points.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<Points pointsList={data.points} />
