<script>
	import { createEventDispatcher } from 'svelte';
    import { range } from '$lib/js/util.js';
	import Icon from '@iconify/svelte';
	import Button from '../../../form/Button.svelte';

	export let name;
	export let channels = new Array(16).fill(true);

	let dispatch = createEventDispatcher();

	function toggleChannel(channelId) {
		const newVal = !channels[channelId];
		// channels[channelId] = newVal;
		dispatch('change', { id: channelId, value: newVal });
	}

    function allChannelsOn() {
        for(let i = 0; i < channels.length; ++i) {
            // channels[i] = true;
            dispatch('change', { id: i, value: true });
        }
    }

    function allChannelsOff() {
        for(let i = 0; i < channels.length; ++i) {
            // channels[i] = false;
            dispatch('change', { id: i, value: false });
        }
    }
</script>

<div class="flex flex-col gap-2">
    <div class="flex flex-row gap-4">
	    <div class="grow overflow-hidden text-ellipsis text-nowrap">{name}</div>
        <div class="flex flex-row gap-1">
            <Button rounded="left" on:click={allChannelsOn}><span class="text-nowrap">All On</span></Button>
            <Button rounded="right" on:click={allChannelsOff}><span class="text-nowrap">All Off</span></Button>
        </div>
    </div>
	<div class="flex flex-row gap-1 flex-wrap place-content-end">
		{#each range(0,8) as i}
			<button
				on:click={() => toggleChannel(i)}
				class="grid h-9 w-9 place-items-center rounded-full border border-solid border-slate-700 bg-slate-800 px-2 py-1 text-slate-300">
				<span class:hidden={!channels[i]}><Icon icon="wpf:checkmark" /></span>
			</button>
		{/each}
	</div>
	<div class="flex flex-row gap-1 flex-wrap place-content-end">
		{#each range(8,16) as i}
			<button
				on:click={() => toggleChannel(i)}
				class="grid h-9 w-9 place-items-center rounded-full border border-solid border-slate-700 bg-slate-800 px-2 py-1 text-slate-300">
				<span class:hidden={!channels[i]}><Icon icon="wpf:checkmark" /></span>
			</button>
		{/each}
	</div>
</div>
