<script lang="ts">
	import type { PageData } from './$types';
	import { ExclamationTriangle } from 'radix-icons-svelte';
	import * as Card from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import * as Alert from '$lib/components/ui/alert';
	import { superForm } from 'sveltekit-superforms/client';
	import { goto } from '$app/navigation';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
	const { form, errors, submitting, enhance } = superForm(data.form, { taintedMessage: false });
</script>

<svelte:head>
	<title>Login | Maccas</title>
</svelte:head>

<Card.Root>
	<Card.Header>
		<Card.Title>Login</Card.Title>
	</Card.Header>
	<form method="POST" use:enhance>
		<Card.Content>
			<div class="grid w-full items-center gap-4">
				<Label for="username">Username</Label>
				<Input
					id="username"
					type="username"
					placeholder="Username"
					name="username"
					aria-invalid={$errors.username ? 'true' : undefined}
					bind:value={$form.username}
				/>
				<Label for="password">Password</Label>
				<Input
					id="password"
					type="password"
					placeholder="Password"
					name="password"
					aria-invalid={$errors.password ? 'true' : undefined}
					bind:value={$form.password}
				/>
			</div>
		</Card.Content>
		<Card.Footer class="w-full">
			<div class="w-full">
				{#if $errors.password || $errors.password}
					<div class="mb-4 w-full">
						<Alert.Root variant="destructive">
							<ExclamationTriangle class="h-4 w-4" />
							<Alert.Title>Error</Alert.Title>
							{#each $errors?.username ?? [] as error}
								<Alert.Description>{error}</Alert.Description>
							{/each}
							{#each $errors?.password ?? [] as error}
								<Alert.Description>{error}</Alert.Description>
							{/each}
						</Alert.Root>
					</div>
				{/if}
				<div class="flex justify-between">
					<Button type="submit" disabled={$submitting} aria-disabled={$submitting}>Login</Button>
					<Button
						variant="outline"
						on:click={async (e) => {
							goto('/register');
						}}
						disabled={$submitting}
						aria-disabled={$submitting}>Register</Button
					>
				</div>
			</div>
		</Card.Footer>
	</form>
</Card.Root>
