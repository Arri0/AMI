<script>
	import { createEventDispatcher } from 'svelte';
    import { openFileBrowser } from '$lib/js/app.js';
	import InputText from '../../form/InputText.svelte';

	export let name;
	export let value = '';
    export let allowedExtensions = undefined;

	let dispatch = createEventDispatcher();

    async function spawnFileBrowser() {
        const file = await openFileBrowser({
            title: 'Select SounFont File',
            buttonText: 'Select',
            path: 'samples:',
            allowedExtensions,
        });
        if(file !== null) {
            value = file;
            dispatch('change', file);
        }
    }
</script>

<div class="flex flex-row gap-4">
	<div class="grow overflow-hidden text-ellipsis text-nowrap">{name}</div>
	<div class="flex flex-row gap-1">
        <InputText bind:value class="rounded-l-full" />
        <button on:click={spawnFileBrowser} class="inline-block align-middle rounded-r-full border border-solid border-slate-700 bg-slate-800 pl-3 pr-4 py-1 text-slate-400 text-sm hover:text-slate-300 transition-colors ease-out">Select</button>
	</div>
</div>
