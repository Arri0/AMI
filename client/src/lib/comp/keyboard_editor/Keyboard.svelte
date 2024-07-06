<script>
	import { createEventDispatcher, onMount } from 'svelte';

	const WKEY_W = 2; // white key width
	const WKEY_H = 12; // white key height
	const WKEY_S = 1 / 8; // white key spacing
	const BKEY_W = 1.4; // black key width
	const BKEY_H = 7.5; // black key height

	const DEFAULT_NUM_KEYS = 128;

	export let startKey = 0;
	export let activeKeys = new Array(DEFAULT_NUM_KEYS).fill(false);
	export let enabledKeys = new Array(DEFAULT_NUM_KEYS).fill(true);
	export let maskColors = new Array(DEFAULT_NUM_KEYS).fill('rgb(107 114 128)');
	export let showMaskButtons = true;

	export const scrollToMiddle = function scrollToMiddle() {
		container.scrollLeft = Math.floor((container.scrollWidth - container.clientWidth) / 2);
	};

	const dispatch = createEventDispatcher();
	let container;

	onMount(() => {
		scrollToMiddle();
	});

	function isBlackKey(index) {
		// prettier-ignore
		return [
			false, true, false, true, false, false, true, false, true, false, true, false
		][index % 12];
	}

	function getKeyLetter(index) {
		// prettier-ignore
		return [
			'C', undefined, 'D', undefined, 'E',
			'F', undefined, 'G', undefined, 'A', undefined, 'H'
		][index % 12];
	}

	function getKeyName(index) {
		if (index < 24) return '';
		index -= 24;
		const octave = Math.floor(index / 12) + 1;
		return `${getKeyLetter(index)}${octave}`;
	}

	function getWhiteKeyIndex(index) {
		return [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6][index % 12];
	}

	function getBlackKeyOffset(index) {
		const offs = BKEY_W / 5;
		// prettier-ignore
		return [
			undefined, -offs, undefined, offs, undefined,
			undefined, -offs, undefined, 0, undefined, offs, undefined,
		][index % 12];
	}

	function getKeyOffset(index) {
		const octaveWidth = (WKEY_W + WKEY_S) * 7;
		const octave = Math.floor(index / 12);
		const whiteKeyOffset = octave * octaveWidth + getWhiteKeyIndex(index) * (WKEY_W + WKEY_S);
		if (isBlackKey(index)) {
			return whiteKeyOffset + WKEY_W - BKEY_W / 2 + getBlackKeyOffset(index, WKEY_W);
		} else {
			return whiteKeyOffset;
		}
	}

	function maskButtonClicked(index) {
		dispatch('mask-button-clicked', index);
	}

	function maskButtonDoubleClicked(index) {
		dispatch('mask-button-double-clicked', index);
	}

	function keyPressed(index) {
		dispatch('key-pressed', index);
	}

	function keyReleased(index) {
		dispatch('key-released', index);
	}
</script>

<!-- prettier-ignore -->
<div class="scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent overflow-x-auto bg-slate-950 select-none" bind:this={container}>
	{#if showMaskButtons}
		<div class="h-4 flex flex-row" style:height="{(WKEY_W + WKEY_S) * 7/12}rem">
			{#each { length: Math.min(activeKeys.length, enabledKeys.length, maskColors.length) } as _, n}
				<div class="grid place-items-center aspect-square shrink-0" style:width="{(WKEY_W + WKEY_S) * 7/12}rem">
					<button class="rounded-full aspect-square" style:background-color="{maskColors[n]}" style:width="{(WKEY_W + WKEY_S) * 7/12*0.55}rem" on:click={() => maskButtonClicked(n)} on:dblclick={() => maskButtonDoubleClicked(n)}></button>
				</div>
			{/each}
		</div>
	{/if}
	<div class="relative">
		<div class="invisible" style:width="{WKEY_W}rem" style:height="{WKEY_H+WKEY_S}rem">
			<!-- This dummy key div serves to properly size the parent container -->
		</div>
		{#each { length: Math.min(activeKeys.length, enabledKeys.length, maskColors.length) } as _, n}
			{@const i = n + startKey}
			{#if isBlackKey(i)}
				<button class="group absolute flex flex-col justify-end bg-slate-950 text-slate-300 items-center z-20 data-[active=false]:data-[enabled=false]:bg-slate-800 data-[active=true]:bg-green-600 border-solid border-slate-950" data-enabled={enabledKeys[n]} data-active={activeKeys[n]} style:width="{BKEY_W}rem" style:height="{BKEY_H}rem" style:top="{WKEY_S}rem" style:left="{getKeyOffset(i)}rem" style:border-width="0 {WKEY_S}rem {WKEY_S}rem {WKEY_S}rem" on:pointerdown={() => keyPressed(i)} on:pointerup={() => keyReleased(i)}>
					<div class="text-2xs/[1.5rem] group-data-[enabled=false]:hidden">{i}</div>
				</button>
			{:else}
				<button class="group absolute flex flex-col justify-end bg-slate-100 text-slate-700 items-center z-10 data-[active=false]:data-[enabled=false]:bg-slate-800 data-[active=true]:bg-green-600" data-enabled={enabledKeys[n]} data-active={activeKeys[n]} style:width="{WKEY_W}rem" style:height="{WKEY_H}rem" style:top="{WKEY_S}rem" style:left="{getKeyOffset(i)}rem" on:pointerdown={() => keyPressed(i)} on:pointerup={() => keyReleased(i)}>
					<span class="text-xs group-data-[enabled=false]:hidden">{getKeyName(i)}</span>
					<span class="text-2xs/[1.5rem] group-data-[enabled=false]:hidden">{i}</span>
				</button>
			{/if}
		{/each}
	</div>
</div>
