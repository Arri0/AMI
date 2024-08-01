<script>
	import { createEventDispatcher } from 'svelte';
	import Icon from '@iconify/svelte';
	import Header from './Header.svelte';

	const dispatch = createEventDispatcher();

	function onGoBack() {
		dispatch('go-back');
	}
</script>

<Header>
	<div class="flex flex-row">
		<div class="w-8">
			<button on:click={onGoBack}
				><Icon icon="icon-park-solid:back" class="inline-block align-middle" /></button>
		</div>
		<div class="grow">
			<slot></slot>
		</div>
		<div class="w-8"></div>
	</div>
</Header>
