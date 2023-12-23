<script lang="ts">
	import type { PageData } from './$houdini';
	export let data: PageData;

	$: ({ Index } = data);
</script>

<svelte:head>
	<title>Maccas</title>
</svelte:head>

{#if $Index.data}
	<div class="grid grid-flow-row gap-4 container mx-auto p-4">
		{#each $Index.data.offers as offer}
			<div class="border rounded p-4 grid grid-flow-row gap-4">
				{JSON.stringify(offer)}
			</div>
		{/each}
	</div>
{/if}
