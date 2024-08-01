<script>
	import { createEventDispatcher } from 'svelte';
	import Icon from '@iconify/svelte';

	export let name;
	export let value = false;

	let dispatch = createEventDispatcher();

	function toggle() {
		value = !value;
		dispatch('change', value);
	}
</script>

<div class="flex flex-row gap-4">
	<div class="grow overflow-hidden text-ellipsis text-nowrap">{name}</div>
	<div>
		<!-- <input
			type="checkbox"
			bind:checked={value}
			on:change={(e) => dispatch('change', e.target.checked)}
			class="scale-150" /> -->
		<button on:click={toggle} class="w-9 h-9 rounded-full grid place-items-center border border-solid border-slate-700 bg-slate-800 px-2 py-1 text-slate-300">
			<span class:hidden={!value}><Icon icon="wpf:checkmark" /></span>
		</button>
	</div>
</div>
