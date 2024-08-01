<script>
	export let value;
	let clazz = '';
	export {clazz as class};
</script>

<input
	{...$$restProps}
	type="text"
	on:change
	bind:value={value}
	class="outline-none text-align-inherit inline-block border border-solid border-slate-700 bg-slate-800 px-2 py-1 text-sm text-slate-300 {clazz}" />
