<script lang="ts">
	import { useMutation } from '@sveltestack/svelte-query';
	import { Skeleton } from './ui/skeleton';
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { writable } from 'svelte/store';
	import type { GetAccountCode$result } from '$houdini';

	export let accountId: string;
	export let removeSelf: () => void;

	const code = writable<string | null>('');
	const getAccountCode = useMutation(
		`add-${accountId}`,
		async () =>
			await fetch(`/api/accounts/${accountId}`, { method: 'GET' })
				.then((r) => r.json())
				.then((j) => j as GetAccountCode$result['pointsByAccountId']),

		{
			onError: () => {
				code.update(() => 'Something went wrong');
			},
			onSuccess: (data) => {
				code.update(() => data.code);
			}
		}
	);

	const refreshAccountCode = useMutation(
		`add-${accountId}`,
		async () =>
			await fetch(`/api/accounts/${accountId}`, { method: 'GET' })
				.then((r) => r.json())
				.then((j) => j as GetAccountCode$result['pointsByAccountId']),
		{
			onError: () => {
				code.update(() => 'Something went wrong');
			},
			onSuccess: (data) => {
				code.update(() => data.code);
			}
		}
	);

	onMount(async () => {
		await $getAccountCode.mutateAsync();
	});
</script>

{#if $getAccountCode.isLoading}
	<Skeleton class="h-[54px] w-full rounded-sm bg-primary/50" />
{:else}
	<div
		class="flex flex-grow flex-row items-center justify-between rounded-sm bg-slate-700/10 p-3 text-center"
	>
		<div class="font-mono">{$refreshAccountCode.data?.code ?? $code}</div>
		<div>
			<Button
				class="material-symbols-outlined cursor-pointer"
				disabled={$getAccountCode.isLoading || $refreshAccountCode.isLoading}
				on:click={async (e) => {
					e.stopPropagation();
					await $refreshAccountCode.mutateAsync();
				}}>refresh</Button
			>
			<Button
				class="material-symbols-outlined cursor-pointer"
				disabled={$getAccountCode.isLoading || $refreshAccountCode.isLoading}
				on:click={async (e) => {
					e.stopPropagation();
					removeSelf();
				}}>close</Button
			>
		</div>
	</div>
{/if}
