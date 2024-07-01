<script>
	import { getApi, cache } from '$lib/js/app.js';
	import Icon from '@iconify/svelte';
	import Header from './comp/Header.svelte';
	import Section from './comp/Section.svelte';
	import Content from './comp/Content.svelte';
	import InputText from '../form/InputText.svelte';
	import Button from '../form/Button.svelte';

	let tempoBpm = 120;

	$: period = 60.0/tempoBpm;
	$: drumMachine = $cache.drum_machine;
	$: enabled = drumMachine.enabled;
	$: numBeats = drumMachine.rhythm.num_beats;
	$: numDivs = drumMachine.rhythm.num_divs;
	$: tempoBpm = drumMachine.tempo_bpm;
	$: voices = drumMachine.voices;

	async function toggle() {
		await getApi().drumMachineSetEnabled(!enabled);
	}

	async function test() {
		console.log('testing...');
		getApi().drumMachineSetEnabled(true);
	}
</script>


<Section>
	<Header>
		<span class="inline-block align-middle mx-2">Drum Machine</span>
		<Icon icon="lucide:drum" class="inline-block align-middle" />
	</Header>
	<Content centerY={true}>
		<div class="mx-auto my-4 flex max-w-[30rem] select-none flex-col gap-2 px-2 items-center w-full">
			<!-- <button on:click={test} class="bg-slate-500">Test</button> -->
			<button on:click={toggle} class="text-9xl {enabled ? 'border-slate-400 text-slate-400' : 'border-slate-700 text-slate-700'} border-2 bg-slate-900 rounded-3xl p-8">
				<div class="{enabled ? 'animate-flip' : ''} animate-infinite" style:animation-duration={`${period}s`} ><Icon icon="mdi:metronome" /></div>
			</button>
		</div>
		<div class="flex flex-row gap-1 text-center">
			<span class="w-12"><InputText rounded="left" value={numBeats} /></span>
			<span class="w-12"><InputText rounded="none" value={numDivs} /></span>
			<span class="text-left overflow-hidden text-nowrap w-24"><Button rounded="right">Set Rhythm</Button></span>
		</div>
	</Content>
</Section>