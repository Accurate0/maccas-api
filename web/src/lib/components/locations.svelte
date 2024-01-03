<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import type { LocationByText$result } from '$houdini';
	import { toast } from 'svelte-sonner';
	import { writable } from 'svelte/store';
	import type { UpdateLocationBody } from '../../routes/api/location/schema';

	type Config = { storeName: string | null; storeId: string | null } | null;
	export let config: Config;

	const configStore = writable<Config>(config);
	export let storeName = $configStore?.storeName;

	let disabled = false;
	let optionsDisabled = false;
	let query: string = '';
	let options: LocationByText$result['location']['text'] = [];

	const searchLocations = async () => {
		if (query) {
			disabled = true;
			const response = await fetch(`/api/location?query=${query}`, { method: 'GET' });
			options = (await response.json()) as LocationByText$result['location']['text'];
			disabled = false;
		} else {
			toast.error('Gotta type something');
		}
	};

	const setLocation = async (storeId: string, newStoreName: string) => {
		optionsDisabled = true;
		const body: UpdateLocationBody = { storeId };
		const response = await fetch(`/api/location`, { method: 'POST', body: JSON.stringify(body) });
		configStore.set({ storeName: newStoreName, storeId });
		storeName = newStoreName;

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
			<div class="grid w-full items-center gap-4 pt-4">
				{#each options as location}
					<Button
						variant="outline"
						disabled={optionsDisabled}
						aria-disabled={optionsDisabled}
						on:click={() => setLocation(location.storeNumber, location.name)}
					>
						{location.name}
					</Button>
				{/each}
			</div>
		{/if}
	</Card.Content>
	<Card.Footer>
		<Button class="w-full" type="submit" on:click={searchLocations} {disabled}>Find</Button>
	</Card.Footer>
</Card.Root>
