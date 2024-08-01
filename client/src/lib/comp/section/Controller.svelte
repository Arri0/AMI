<script>
	import { getApi, cache, beatState } from '$lib/js/app.js';
	import Icon from '@iconify/svelte';
	import Header from './comp/Header.svelte';
	import Section from './comp/Section.svelte';
	import Content from './comp/Content.svelte';
	import InputText from '../form/InputText.svelte';
	import Button from '../form/Button.svelte';

	let flipTempoIcon = false;
	let tempoBpm = 90;
	let numBeats = 0;
	let numDivs = 0;

	let prevBeat = -1;

	$: {
		const beat = $beatState.beat;
		if(beat != prevBeat) {
			prevBeat = beat;
			flipTempoIcon = !flipTempoIcon
		}
	};

	$: controller = $cache.controller;
	$: enabled = controller.enabled;
	$: tempoBpm = controller.tempo_bpm;
	$: numBeats = controller.rhythm.num_beats;
	$: numDivs = controller.rhythm.num_divs;

	async function toggle() {
		console.log(await getApi().controllerSetEnabled(!enabled));
	}

	async function setRhythm(numBeats, numDivs) {
		numBeats = parseInt(numBeats);
		numDivs = parseInt(numDivs);
		if(Number.isNaN(numBeats) || Number.isNaN(numDivs))
			return;
		console.log(await getApi().controllerSetRhythm(numBeats, numDivs));
	}

	async function setTempoBpm(tempoBpm) {
		tempoBpm = parseFloat(tempoBpm);
		if(Number.isNaN(tempoBpm))
			return;
		console.log(await getApi().controllerSetTempoBpm(tempoBpm));
	}
</script>


<Section>
	<Header>
		<span class="inline-block align-middle mx-2">Controller</span>
		<Icon icon="lucide:drum" class="inline-block align-middle" />
	</Header>
	<Content centerY={true}>
		<div class="flex flex-col gap-4">
			<div class="mx-auto my-4 flex max-w-[30rem] select-none flex-col gap-2 px-2 items-center w-full place-content-center">
				<!-- <button on:click={test} class="bg-slate-500">Test</button> -->
				<button on:click={toggle} class="text-9xl {enabled ? 'border-slate-400 text-slate-400' : 'border-slate-700 text-slate-700'} border-2 bg-slate-900 rounded-3xl p-8">
					<div class:flip-x={flipTempoIcon}><Icon icon="mdi:metronome" /></div>
				</button>
				<div class="flex flex-row gap-1 text-center">
					<InputText class="select-none pointer-events-none rounded-l-full w-12" readonly={true} value={$beatState.beat} />
					<InputText class="select-none pointer-events-none rounded-r-full w-12" readonly={true} value={$beatState.div} />
				</div>
			</div>
			<div class="flex flex-row gap-1 text-center place-content-center">
				<InputText class="rounded-l-full w-12" value={numBeats} on:change={e => numBeats = e.target.value} />
				<InputText class="w-12" value={numDivs} on:change={e => numDivs = e.target.value} />
				<Button on:click={() => setRhythm(numBeats, numDivs)} class="rounded-r-full text-left overflow-hidden text-nowrap w-24">Set Rhythm</Button>
			</div>
			<div class="flex flex-row gap-1 text-center place-content-center">
				<InputText class="rounded-l-full w-16" value={tempoBpm} on:change={e => tempoBpm = e.target.value} />
				<Button on:click={() => setTempoBpm(tempoBpm)} class="rounded-r-full text-left overflow-hidden text-nowrap w-24">Set Tempo</Button>
			</div>
		</div>
	</Content>
</Section>