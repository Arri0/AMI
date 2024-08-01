<script>
	export let active = false;
	let clazz = '';
	export { clazz as class };
</script>

<button
	{...$$restProps}
	on:click
	class="inline-block outline-none border border-solid {active ? 'border-slate-500 text-slate-300':'border-slate-700 text-slate-400'} bg-slate-800 px-2 py-1 text-sm hover:text-slate-300 transition-colors ease-out read-only:outline-none {clazz}">
	<slot></slot>
</button>
