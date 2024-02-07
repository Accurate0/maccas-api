<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Switch } from '$lib/components/ui/switch';
	import { toast } from 'svelte-sonner';
	import type { PageData } from './$types';

	export let data: PageData;

	const toggleActive = async (activate: boolean, userId: string) => {
		const response = await fetch(`/api/users/${userId}/active`, {
			method: activate ? 'POST' : 'DELETE'
		});
		if (response.status === 204) {
			toast('Account updated');
		} else {
			toast.error(await response.text());
		}
	};
</script>

<div class="grid grid-flow-row gap-4">
	{#await data.users}
		{#each Array(5) as _}
			<Card.Root>
				<div class="flex">
					<Card.Header class="grid w-full">
						<Skeleton class="h-[22px] w-[33%] rounded-xl" />
					</Card.Header>
					<Card.Header>
						<Skeleton class="h-[40px] w-[40px] rounded-xl" />
					</Card.Header>
				</div>
			</Card.Root>
		{/each}
	{:then users}
		{#each users as user}
			<Card.Root>
				<div class="flex">
					<Card.Header class="grid w-full">
						{user.username}
					</Card.Header>
					<Card.Header>
						<Switch
							id="active"
							checked={user.active}
							onCheckedChange={(activate) => toggleActive(activate, user.id)}
						/>
						<Label for="active">Active</Label>
					</Card.Header>
				</div>
			</Card.Root>
		{/each}
	{/await}
</div>
