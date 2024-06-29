<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Switch } from '$lib/components/ui/switch';
	import { toast } from 'svelte-sonner';
	import type { PageData } from './$types';
	import Button from '$lib/components/ui/button/button.svelte';
	import { ExclamationTriangle, Check, Cross1 } from 'radix-icons-svelte';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import { formatDistanceToNow } from 'date-fns/formatDistanceToNow';

	export let data: PageData;

	const resetRateLimit = async () => {
		const response = await fetch(`/api/ratelimit`, {
			method: 'DELETE'
		});

		if (response.status === 204) {
			toast('Rate limit reset');
		} else {
			toast.error(await response.text());
		}
	};

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
	<Card.Root>
		<Card.Header>Actions</Card.Header>
		<Card.Content>
			<Button on:click={resetRateLimit}>Reset rate limit</Button>
		</Card.Content>
	</Card.Root>

	{#await data.notifications}
		<Card.Root>
			<div class="m-4 grid grid-flow-row gap-4">
				<h4 class="text-sm font-semibold">Recent notifications</h4>
			</div>

			<div class="flex h-96 flex-col overflow-y-scroll">
				<Skeleton class="m-4 mt-0 h-96 w-[inherit] rounded-xl" />
			</div>
		</Card.Root>
	{:then notifications}
		<Card.Root>
			<div class="m-4 grid grid-flow-row gap-4">
				<h4 class="text-sm font-semibold">Recent notifications</h4>
			</div>

			<div class="flex max-h-96 flex-col overflow-y-scroll">
				{#each notifications as notification}
					<Alert.Root variant="default" class="m-4 mt-0 w-[inherit]">
						{#if notification.priority === 'HIGH'}
							<ExclamationTriangle class="h-4 w-4" />
						{:else if notification.type === 'USER_CREATED' || notification.type === 'USER_ACTIVATED'}
							<Check class="h-4 w-4" />
						{:else if notification.type === 'USER_DEACTIVATED'}
							<Cross1 class="h-4 w-4" />
						{/if}
						<Alert.Title>{notification.content}</Alert.Title>
						<Alert.Description>{formatDistanceToNow(notification.createdAt)} ago</Alert.Description>
					</Alert.Root>
				{/each}
			</div>
		</Card.Root>
	{/await}

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
						<div class="text-sm">
							Roles: {user.roles.join(', ')}
						</div>
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
