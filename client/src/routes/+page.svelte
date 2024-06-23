<script>
	import { appInit, appDestroy, getApi } from '../lib/js/app.js';
	import { onDestroy, onMount } from 'svelte';
	import { openFileBrowser } from '../lib/js/app';
	import Keyboard from '../lib/comp/section/comp/Keyboard.svelte';
	import MainMenu from '../lib/comp/section/comp/MainMenu.svelte';
	import MidiPorts from '../lib/comp/section/MidiPorts.svelte';
	import Nodes from '../lib/comp/section/Nodes.svelte';
	import DrumMachine from '../lib/comp/section/DrumMachine.svelte';
	import Log from '../lib/comp/section/Log.svelte';
	import Settings from '../lib/comp/section/Settings.svelte';
	import FileBrowser from '../lib/comp/file_browser/FileBrowser.svelte';

	let activeKeys;
	let currentSection;
	let previousSection;
	let currentNode = null;

	function onSectionChanged() {
		if (previousSection === currentSection && currentSection === 'nodes') {
			currentNode = null;
		}
	}

	onMount(() => {
		appInit();

		const api = getApi();

		api.addEventListener('midi', (ev) => {
			const channel = ev.detail.channel;
			const kind = ev.detail.kind;
			console.log('Midi event from', channel, ':', kind);
			if ('NoteOn' in kind) {
				const midi_ev = kind.NoteOn;
				const note = midi_ev.note;
				const velocity = midi_ev.velocity;
				if (activeKeys !== undefined) activeKeys[note] = velocity > 0;
			} else if ('NoteOff' in kind) {
				const midi_ev = kind.NoteOff;
				const note = midi_ev.note;
				if (activeKeys !== undefined) activeKeys[note] = false;
			}
		});

		api.addEventListener('connected', async () => {
			// const res = await openFileBrowser({
			//     title: 'Open File',
			//     buttonText: 'Open',
			//     path: 'samples:',
			//     allowedExtensions: ['sf2', 'sfz'],
			// });
			// console.log('selected-file:', res);
		});
	});

	onDestroy(() => {
		appDestroy();
	});
</script>

<div class="grid-rows-1fr-auto grid h-screen w-screen bg-slate-900">
	{#if currentSection === 'midi-ports'}
		<MidiPorts />
	{:else if currentSection === 'nodes'}
		<Nodes bind:currentNode />
	{:else if currentSection === 'drum-machine'}
		<DrumMachine />
	{:else if currentSection === 'log'}
		<Log />
		<Keyboard
			showMaskButtons={true}
			bind:activeKeys
			on:key-pressed={(ev) => console.log('pressed', ev.detail)}
			on:key-released={(ev) => console.log('released', ev.detail)} />
	{:else if currentSection === 'settings'}
		<Settings />
	{/if}
	<MainMenu bind:currentSection bind:previousSection on:section-changed={onSectionChanged} />
	<FileBrowser />
</div>
