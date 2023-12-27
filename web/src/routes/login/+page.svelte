<script lang="ts">
	import type { ActionData } from './$types';

	import { ExclamationTriangle } from 'radix-icons-svelte';
	import * as Card from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import * as Alert from '$lib/components/ui/alert';

	export let form: ActionData;
</script>

<svelte:head>
	<title>Login | Maccas</title>
</svelte:head>

<Card.Root>
	<Card.Header>
		<Card.Title>Login</Card.Title>
	</Card.Header>
	<form method="POST">
		<Card.Content>
			<div class="grid w-full items-center gap-4">
				<Label for="username">Username</Label>
				<Input id="username" type="username" placeholder="Username" name="username" />
				<Label for="password">Password</Label>
				<Input id="password" type="password" placeholder="Password" name="password" />
			</div>
		</Card.Content>
		<Card.Footer class="w-full">
			<div class="w-full">
				{#if form?.error}
					<div class="mb-4 w-full">
						<Alert.Root variant="destructive">
							<ExclamationTriangle class="h-4 w-4" />
							<Alert.Title>Error</Alert.Title>
							<Alert.Description>{form.error}</Alert.Description>
						</Alert.Root>
					</div>
				{/if}

				<Button type="submit">Login</Button>
			</div>
		</Card.Footer>
	</form>
</Card.Root>
