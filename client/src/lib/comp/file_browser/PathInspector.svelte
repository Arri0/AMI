<script>
    import { createEventDispatcher } from 'svelte';
	import Icon from '@iconify/svelte';

    export let path;
    let dispatch = createEventDispatcher();

    $: pathComponents = path?.split(/[\\/]/) ?? [];

    function onComponentClicked(index) {
        dispatch('path-selected', pathComponents.slice(0, index+1).join('/'));
    }
</script>

<div class="px-2 py-4 flex flex-row gap-1 border-b-slate-900 border-b-2 text-slate-400">
    {#each pathComponents as comp,i}
        <button on:click={() => onComponentClicked(i)} class="bg-slate-900 py-2 px-4 first:rounded-l-full first:border-l-2 border-t-2 border-b-2 last:border-r-2 border-slate-800 last:rounded-r-full">
            {#if i == 0}
                <Icon icon="ion:home" class="inline-block align-middle mr-2" />
            {/if}
            <span class="inline-block align-middle">{comp}</span>
        </button>
    {/each}
</div>