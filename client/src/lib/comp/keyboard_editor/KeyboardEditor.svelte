<script>
	import { registerKeyboardEditor } from '$lib/js/app.js';
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import Header from '../section/comp/Header.svelte';
	import Keyboard from './Keyboard.svelte';
	import Button from '../form/Button.svelte';
	import { getApi } from '../../js/app';

	const DEFAULT_NUM_KEYS = 128;

	let hidden = true;
	let keyboardHidden = true;
	let title = '';
	let activeKeys;
	let enabledKeys;
	let mask;
	let maskColors;
	let cbResolve = null;
	let mode = null;
	let prevMaskId = null;

	$: maskColors = mask?.map((x) => (x ? 'rgb(101 163 13)' : 'rgb(107 114 128)'));

	async function open(params) {
		keyboardHidden = false;
		title = params.title ?? 'Keyboard Editor';
		mask = params.mask ?? new Array(DEFAULT_NUM_KEYS).fill(true);
		activeKeys = new Array(mask.length).fill(false);
		enabledKeys = new Array(mask.length).fill(true);
		getApi().addEventListener('midi', onMidi);
		hidden = false;
		return new Promise((resolve, _) => {
			cbResolve = resolve;
		});
	}

	function close() {
		getApi().removeEventListener('midi', onMidi);
		hidden = true;
		keyboardHidden = true;
		cbResolve(mask);
		cbResolve = null;
	}

	function maskAllOn() {
		mask.fill(true);
		mask = mask;
	}

	function maskAllOff() {
		mask.fill(false);
		mask = mask;
	}

	function maskInvertAll() {
		mask = mask.map((x) => !x);
	}

	function maskButtonClicked(id) {
		if (mode === null) {
			mask[id] = !mask[id];
		} else if (mode === 'region' || mode === 'negative-region') {
			if (prevMaskId === null) {
				prevMaskId = id;
				maskColors = maskColors.map((_, i) => (i == id ? 'rgb(220 38 38)' : 'rgb(107 114 128)'));
			} else {
				selectMaskRegion(Math.min(prevMaskId, id), Math.max(prevMaskId, id), mode === 'region');
				prevMaskId = null;
			}
		} else if (mode === 'left-half') {
			selectMaskLeftHalf(id);
		} else if (mode === 'right-half') {
			selectMaskRightHalf(id);
		}
	}

	function selectMaskRegion(start, end, value) {
		mask = mask.map((_, i) => (i >= start && i <= end ? value : !value));
	}

	function selectMaskLeftHalf(middle) {
		mask = mask.map((_, i) => i <= middle);
	}

	function selectMaskRightHalf(middle) {
		mask = mask.map((_, i) => i >= middle);
	}

	function toggleMode(name) {
		prevMaskId = null;
		mode = mode == name ? null : name;
	}

	function maskButtonDoubleClicked(id) {
		mask = mask.map((_, i) => i <= id);
	}

	function onMidi(ev) {
		const kind = ev.detail.kind;
		if (typeof kind === 'object' && !Array.isArray(kind) && kind !== null) {
			if ('NoteOn' in kind) {
				const id = kind.NoteOn.note;
				maskButtonClicked(id);
			}
		}
	}

	onMount(() => {
		registerKeyboardEditor(open);
	});
</script>

<div
	class="grid-rows-auto-1fr fixed inset-0 z-[9999] grid grid-cols-1 gap-4 bg-slate-950 text-slate-300"
	class:hidden>
	<Header>
		<div class="flex flex-row">
			<div class="w-8"></div>
			<div class="grow text-center">
				<span class="mx-2 inline-block align-middle">{title}</span>
			</div>
			<div class="w-8">
				<button on:click={close}><Icon icon="fa:close" class="inline-block align-middle" /></button>
			</div>
		</div>
	</Header>
	<div class="flex flex-col gap-4">
		<div
			class="scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent flex flex-row gap-1 overflow-x-auto text-nowrap px-4">
			<span class="w-auto"><Button rounded="left" on:click={maskAllOn}>All On</Button></span>
			<span class="w-auto"><Button rounded="none" on:click={maskAllOff}>All Off</Button></span>
			<span class="w-auto"
				><Button rounded="right" on:click={maskInvertAll}>Invert All</Button></span>
			<span class="mx-2 border border-l-2 border-slate-700"></span>
			<span class="w-auto"
				><Button
					rounded="left"
					active={mode == 'left-half'}
					on:click={() => toggleMode('left-half')}>Left Half</Button
				></span>
			<span class="w-auto"
				><Button
					rounded="none"
					active={mode == 'right-half'}
					on:click={() => toggleMode('right-half')}>Right Half</Button
				></span>
			<span class="w-auto"
				><Button rounded="none" active={mode == 'region'} on:click={() => toggleMode('region')}
					>Region</Button
				></span>
			<span class="w-auto"
				><Button
					rounded="right"
					active={mode == 'negative-region'}
					on:click={() => toggleMode('negative-region')}>Negative Region</Button
				></span>
		</div>
		{#if !keyboardHidden}
			<Keyboard
				showMaskButtons={true}
				bind:activeKeys
				bind:enabledKeys
				bind:maskColors
				on:key-pressed={(ev) => console.log('pressed', ev.detail)}
				on:key-released={(ev) => console.log('released', ev.detail)}
				on:mask-button-clicked={(ev) => maskButtonClicked(ev.detail)}
				on:mask-button-double-clicked={(ev) => maskButtonDoubleClicked(ev.detail)} />
		{/if}
	</div>
</div>
