<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import * as Alert from '$lib/components/ui/alert';
	import { ExclamationTriangle } from 'radix-icons-svelte';
</script>

<svelte:head>
	<title>Inactive | Maccas</title>
</svelte:head>

<Card.Root>
	<Card.Header>
		<Alert.Root variant="destructive">
			<ExclamationTriangle class="h-4 w-4" />
			<Alert.Title>Inactive</Alert.Title>
			<Alert.Description>This account is currently inactive</Alert.Description>
		</Alert.Root>
	</Card.Header>
</Card.Root>
