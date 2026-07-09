<script lang="ts">
	import { resolve } from '$app/paths';
	import Code from '$lib/md/code.md';
</script>

<div class="mx-auto max-w-3xl p-8">
	<div class="mb-12">
		<a
			href={resolve('/')}
			class="inline-block rounded-full bg-bg p-4 text-2xl transition-colors hover:bg-bg1"
			aria-label="Back to home"
		>
			←
		</a>
	</div>

	<Code />
</div>
