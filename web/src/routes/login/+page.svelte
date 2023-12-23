<script lang="ts">
	import '@material/web/textfield/outlined-text-field';
	import '@material/web/button/filled-button';

	export let form: import('./$types').ActionData;
</script>

<svelte:head>
	<title>Login | Maccas</title>
</svelte:head>

<div class="flex h-screen justify-center items-center">
	<form method="post">
		<div class="grid grid-flow-row gap-4 w-full max-w-md container mx-auto">
			<h3 class="text-xl font-medium">Sign in</h3>
			<md-outlined-text-field
				class="w-full max-w-sm items-center gap-2"
				label="Username"
				type="username"
				name="username"
				required
			/>
			<md-outlined-text-field
				class="w-full max-w-sm items-center gap-2"
				label="Password"
				type="password"
				name="password"
				required
			/>

			{#if form?.error}
				<p class="outline outline-1 rounded outline-gray-300 p-4">Invalid credentials!</p>
			{/if}

			<md-filled-button class="w-full max-w-sm items-center gap-2">Log in</md-filled-button>
		</div>
	</form>
</div>
