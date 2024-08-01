<script>
	import Icon from '@iconify/svelte';
	import { createEventDispatcher } from 'svelte';

	export let value = 0;
	export let min = null;
	export let max = null;
	export let largeStep = null;
	export let smallStep = null;
	export let unit = null;
	export let defaultValue = null;
	export let numDecimalPlaces = null;
    export let unitWidth = 'auto';

    export let inputValue = sanitizeValue(value);
	$: inputValue = sanitizeValue(value);

	let dispatch = createEventDispatcher();

    function sanitizeValue(value) {
        if(numDecimalPlaces !== null) {
            const coef = Math.pow(10, numDecimalPlaces);
            value = Math.round(value*coef)/coef;
        }
        if(min !== null && value < min)
            value = min;
        if(max !== null && value > max)
            value = max;
        return value;
    }

    function emitChange() {
        if(Number.isNaN(inputValue))
            inputValue = defaultValue;
        inputValue = sanitizeValue(inputValue);
        value = inputValue;
        dispatch('change', value);
    }
</script>

<div class="inline-flex flex-col gap-2 items-end">
	<div class="flex flex-row w-44 gap-1">
		<input
			type="number"
			bind:value={inputValue}
			on:change={() => emitChange()}
			class="outline-none text-align-inherit inline-block align-middle w-full rounded-l-full {unit === null ? 'rounded-r-full' : ''} border border-solid border-slate-700 bg-slate-800 px-2 py-1 text-sm text-slate-300 read-only:outline-none" />
		{#if unit !== null}
			<span class="font-mono text-left inline-block align-middle rounded-r-full border border-solid border-slate-700 bg-slate-800 pl-2 pr-3 py-1 text-slate-400 text-sm" style:width={unitWidth}>{unit}</span>
		{/if}
	</div>
	<div class="flex flex-row text-2xl w-full place-content-between">
		{#if largeStep !== null}
			<button on:click={() => {inputValue -= largeStep; emitChange()}} title="-{largeStep} {unit ?? ''}"><Icon icon="solar:double-alt-arrow-down-bold" /></button>
		{/if}
		{#if smallStep !== null}
			<button on:click={() => {inputValue -= smallStep; emitChange()}} title="-{smallStep} {unit ?? ''}"><Icon icon="mingcute:down-fill" /></button>
		{/if}
		{#if defaultValue !== null}
			<button on:click={() => {inputValue = defaultValue; emitChange()}} title="{defaultValue} {unit ?? ''}"><Icon icon="bx:reset" /></button>
		{/if}
		{#if smallStep !== null}
			<button on:click={() => {inputValue += smallStep; emitChange()}} title="+{smallStep} {unit ?? ''}"><Icon icon="mingcute:up-fill" /></button>
		{/if}
		{#if largeStep !== null}
			<button on:click={() => {inputValue += largeStep; emitChange()}} title="+{largeStep} {unit ?? ''}"><Icon icon="solar:double-alt-arrow-up-bold" /></button>
		{/if}
	</div>
</div>