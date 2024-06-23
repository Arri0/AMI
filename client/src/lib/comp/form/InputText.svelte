<script>
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

<input
	{...$$restProps}
	type="text"
	on:change
	class="outline-none text-align-inherit w-full inline-block {roundedClass} border border-solid border-slate-700 bg-slate-800 px-2 py-1 text-sm text-slate-300 read-only:outline-none" />
