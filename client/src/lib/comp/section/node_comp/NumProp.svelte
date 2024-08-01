<script>
	import NumericField from './NumericField.svelte';

	export let name = '';
	export let value = 0;
	export let min = null;
	export let max = null;
	export let largeStep = null;
	export let smallStep = null;
	export let unit = null;
	export let defaultValue = null;
	export let numDecimalPlaces = null;
    export let unitWidth = 'auto';

</script>

<div class="flex flex-row gap-4">
	<div class="grow overflow-hidden text-ellipsis text-nowrap">{name}</div>
	<NumericField bind:value bind:min bind:max bind:largeStep bind:smallStep bind:unit bind:defaultValue bind:numDecimalPlaces bind:unitWidth on:change />
</div>
