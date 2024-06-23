<script>
	export let options = [];
	export let value;
	export let rounded = 'both';

	let roundedClass = 'rounded-full'

	$: switch(rounded) {
		case 'both':
			roundedClass = 'rounded-full';
			break;
		case 'left':
			roundedClass = 'rounded-l-full';
			break;
		case 'right':
			roundedClass = 'rounded-r-full';
			break;
		default:
			roundedClass = '';
			break;
	}
</script>

<select
	{...$$restProps}
	bind:value
	on:change
	class="w-full {roundedClass} border border-solid border-slate-700 bg-slate-800 px-2 py-1 text-sm text-slate-300">
	{#each options as option}
		{@const [value, text] = Array.isArray(option) ? option : [option, option]}
		<option {value}>{text}</option>
	{/each}
</select>
