<script>
	export let options = [];
	export let value;
	let clazz = '';
	export { clazz as class };
</script>

<select
	{...$$restProps}
	bind:value
	on:change
	class="border border-solid border-slate-700 bg-slate-800 px-2 py-1 inline-block text-sm text-slate-300 box-border {clazz}">
	{#each options as option}
		{@const [value, text] = Array.isArray(option) ? option : [option, option]}
		<option {value}>{text}</option>
	{/each}
</select>
