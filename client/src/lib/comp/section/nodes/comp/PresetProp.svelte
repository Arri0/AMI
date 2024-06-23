
<script>
    import { createEventDispatcher } from 'svelte';
    import Select from '../../../form/Select.svelte';

	export let name = '';
    export let bank = 0;
    export let preset = 0;
    export let presetMap = null;

    let bankOptions;
    let presetOptions;
    let dispatch = createEventDispatcher();

    let bankValue;
    let presetValue;

    $: {presetMap; updateBankOptions(); updatePresetOptions();};
    $: {bank; updatePresetOptions(); updateBank();}
    $: {preset; updatePreset();}

    function updateBankOptions() {
        bankOptions = Object.keys(presetMap?.banks ?? {}).map(x => parseInt(x));
    }

    function updatePresetOptions() {
        const presets = presetMap?.banks?.[bank] ?? {};
        presetOptions = Object.entries(presets).map(([key, preset]) => [parseInt(key), `${key}: ${preset.name}`]);
    }

    function updateBank() {
        bankValue = bank;
        updatePreset();
    }

    function updatePreset() {
        presetValue = preset;
        if(presetOptions.find((p) => p[0] == presetValue) === undefined) {
            if(presetOptions.length > 0) {
                presetValue = presetOptions[0][0];
                emitChange();
            }
        }
    }

    function emitChange() {
        dispatch('change', {
            bank: bankValue,
            preset: presetValue,
        });
    }

</script>

<div class="flex flex-row gap-4 align-middle">
	<div class="grow overflow-hidden text-ellipsis text-nowrap">{name}</div>
    <div class="flex flex-row gap-1 w-64">
        <span><Select options={bankOptions} rounded="left" bind:value={bankValue} on:change={emitChange}></Select></span>
        <span class="grow"><Select options={presetOptions} rounded="right" bind:value={presetValue} on:change={emitChange}></Select></span>
    </div>
</div>
