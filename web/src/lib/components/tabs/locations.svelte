<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import type { LocationByText$result } from '$houdini';
	import { toast } from 'svelte-sonner';
	import type { UpdateLocationBody } from '../../../routes/api/location/schema';
	import { scale, slide } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import { configStore } from '$lib/config';
	import { page } from '$app/stores';

	let disabled = false;
	let optionsDisabled = false;
	let query: string = '';
	let options: LocationByText$result['location']['text'] = [];

	const searchLocations = async () => {
		if (query) {
			disabled = true;
			const response = await fetch(`/api/location?query=${encodeURIComponent(query)}`, {
				method: 'GET'
			});
			options = (await response.json()) as LocationByText$result['location']['text'];
			disabled = false;
		} else {
			toast.error('Gotta type something');
		}
	};

	const setLocation = async (storeId: string, newStoreName: string) => {
		optionsDisabled = true;
		const body: UpdateLocationBody = { storeId };
		const response = await fetch('/api/location', { method: 'POST', body: JSON.stringify(body) });
		configStore.set({ storeName: newStoreName, storeId });

		if (response.ok) {
			toast('Location updated');
		} else {
			toast.error('Something went wrong');
		}
		optionsDisabled = false;
	};
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Search</Card.Title>
	</Card.Header>
	<Card.Content>
		<div class="grid w-full items-center gap-4">
			<Input
				id="query"
				type="username"
				placeholder="e.g. Armadale"
				name="query"
				bind:value={query}
				required
			/>
		</div>
		{#if options.length > 0}
			<div transition:slide>
				<div class="grid w-full items-center gap-4 pt-4">
					{#each options as location (location)}
						<div animate:flip transition:scale>
							<Button
								class="w-full"
								variant="outline"
								disabled={optionsDisabled}
								aria-disabled={optionsDisabled}
								on:click={() => setLocation(location.storeNumber, location.name)}
							>
								{location.name}
							</Button>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</Card.Content>
	<Card.Footer>
		<Button class="w-full" type="submit" on:click={searchLocations} {disabled}>Find</Button>
	</Card.Footer>
</Card.Root>
