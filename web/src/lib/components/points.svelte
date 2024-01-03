<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';

	export let pointsList: Promise<import('$houdini').Index$result['points'] | undefined>;
</script>

<div class="grid grid-flow-row gap-4">
	{#await pointsList then pointsList}
		{#each pointsList ?? [] as { currentPoints }}
			<Card.Root>
				<div class="grid grid-flow-col justify-between">
					<Card.Header class="justify-center">
						<Card.Title>{currentPoints}</Card.Title>
					</Card.Header>
					<Card.Header>
						{#if currentPoints >= 2500}
							<Badge class="h-fit w-fit">
								<span class="material-symbols-outlined">local_cafe</span>
							</Badge>
						{/if}
						{#if currentPoints >= 5000}
							<Badge class="h-fit w-fit">
								<span class="material-symbols-outlined">icecream</span>
							</Badge>
						{/if}
						{#if currentPoints >= 7500}
							<Badge class="h-fit w-fit">
								<span class="material-symbols-outlined">lunch_dining</span>
							</Badge>
						{/if}
					</Card.Header>
				</div>
			</Card.Root>
		{/each}
	{/await}
</div>
