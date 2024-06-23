<script>
    import { createEventDispatcher } from 'svelte';

    export let buttonText;
    export let fileName = '';

    let dispatch = createEventDispatcher();

    function onButtonClicked() {
        if(fileName.trim().length !== 0)
            dispatch('file-selected', fileName);
    }
</script>

<div class="px-2 py-2 flex flex-row border-slate-700 border-t-2 bg-slate-900 text-slate-400">
    <input type="text" class="grow h-8 bg-slate-950 text-sm/8 rounded-l-full px-4" bind:value={fileName} />
    <button on:click={onButtonClicked} class="rounded-r-full bg-slate-800 h-8 text-sm/8 pl-3 pr-4 select-none hover:text-slate-300 transition-colors ease-out">{buttonText}</button>
</div>