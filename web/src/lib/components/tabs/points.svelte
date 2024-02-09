<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { writable, type Writable } from 'svelte/store';
	import { slide } from 'svelte/transition';
	import PointCode from '../point-code.svelte';

	export let pointsList: Promise<import('$houdini').Index$result['points'] | undefined>;
	let state: Writable<Record<string, boolean>> = writable({});

	const addAccount = (accountId: string) => {
		// FIXME: :)
		if (!$state[accountId]) {
			$state[accountId] = true;
		}

		$state = $state;
	};

	const removeAccount = async (accountId: string) => {
		state.update((s) => ({ ...s, [accountId]: false }));
	};
</script>

<div class="grid grid-flow-row gap-4">
	{#await pointsList then pointsList}
		{#each pointsList ?? [] as { currentPoints, accountId }}
			<Card.Root
				on:click={() => {
					addAccount(accountId);
				}}
			>
				<div class="grid grid-flow-col justify-between">
					<Card.Header class="justify-center">
						<Card.Title>{currentPoints}</Card.Title>
					</Card.Header>
					<Card.Header>
						<div>
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
						</div>
					</Card.Header>
				</div>
				{#if $state[accountId]}
					<Card.Footer>
						<div in:slide out:slide class="grid h-full w-full grid-flow-row gap-2">
							{#if $state[accountId]}
								<span in:slide out:slide>
									<PointCode {accountId} removeSelf={() => removeAccount(accountId)} />
								</span>
							{/if}
						</div>
					</Card.Footer>
				{/if}
			</Card.Root>
		{/each}
	{/await}
</div>
