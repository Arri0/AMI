
<script>
    import { createEventDispatcher } from 'svelte';
	import NumericField from './NumericField.svelte';
    import Select from '../../../form/Select.svelte';

	export let name = '';
    export let value = 'Identity';

    let kindValue;
    let minValue;
    let maxValue;

    updateFields(value);
    $: updateFields(value);

    let dispatch = createEventDispatcher();

    function updateFields(newValue) {
        if(newValue === 'Identity') {
            kindValue = 'Identity';
        } else if(typeof value === 'object' && !Array.isArray(value) && value !== null) {
            if('Linear' in value) {
                kindValue = 'Linear';
                minValue = value.Linear.min;
                maxValue = value.Linear.max;
            }
        }
    }

    function emitChange() {
        let newValue;
        switch (kindValue) {
            case 'Identity':
                newValue = 'Identity';
                break;
            case 'Linear':
                newValue = {
                    'Linear': {
                        min: minValue,
                        max: maxValue,
                    }
                }
                break;
            default:
                break;
        }
        console.log('emit change', kindValue, newValue)
        dispatch('change', newValue);
    }

</script>

<div class="flex flex-row gap-4 align-middle">
	<div class="grow overflow-hidden text-ellipsis text-nowrap">{name}</div>
    <div class="flex flex-col gap-2 place-items-end">
        <div class="w-44">
            <Select options={['Identity', 'Linear']} bind:value={kindValue} on:change={emitChange} />
        </div>
        {#if kindValue === 'Linear'}
            <hr class="border border-slate-800 w-full my-2" />
            <div class="flex flex-row g-2">
                <div class="w-12 pt-0.5">Min:</div>
                <NumericField bind:value={minValue} smallStep={1} largeStep={16} defaultValue={0} min={0} max={127} numDecimalPlaces={0} on:change={emitChange} />
            </div>
            <div class="flex flex-row g-2">
                <div class="w-12 pt-0.5">Max:</div>
                <NumericField bind:value={maxValue} smallStep={1} largeStep={16} defaultValue={127} min={0} max={127} numDecimalPlaces={0} on:change={emitChange} />
            </div>
        {/if}
    </div>
</div>
