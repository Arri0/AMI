<script>
	import { getApi, openKeyboardEditor } from '$lib/js/app.js';
	import InputText from '../../form/InputText.svelte';
	import BoolProp from '../node_comp/BoolProp.svelte';
	import FileProp from '../node_comp/FileProp.svelte';
	import FilterChannels from '../node_comp/FilterChannels.svelte';
	import NumProp from '../node_comp/NumProp.svelte';
	import VelocityMapping from '../node_comp/VelocityMapping.svelte';

	export let id;
	export let instance;

	async function changeName(newName) {
		await getApi().renderNodeSetName(id, newName);
	}

	async function changeEnabled(newEnabled) {
		await getApi().renderNodeSetEnabled(id, newEnabled);
	}

	async function changeGlobalTransposition(newEnabled) {
		await getApi().renderNodeSetIgnoreGlobalTransposition(id, !newEnabled);
	}

	async function changeGain(newGain) {
		await getApi().renderNodeSetGain(id, newGain);
	}

	async function changeTransposition(newTransposition) {
		await getApi().renderNodeSetTransposition(id, newTransposition);
	}

	async function changeVelocityMapping(newMapping) {
		await getApi().renderNodeSetVelocityMapping(id, newMapping);
	}

	async function loadFile(newFile) {
		await getApi().renderNodeLoadFile(id, newFile);
	}

	async function updateMidiFilterChannel(channelId, enabled) {
		await getApi().renderNodeUpdateMidiFilterChannel(id, channelId, enabled);
	}

	async function updateMidiFilterSustain(enabled) {
		await getApi().renderNodeUpdateMidiFilterControlChange(id, 64, enabled);
	}

	async function setUserPresetEnabled(presetId, enabled) {
		await getApi().renderNodeSetUserPresetEnabled(id, presetId, enabled);
	}

	async function openNoteMaskEditor() {
		const mask = await openKeyboardEditor({
			title: 'Keyboard Mask',
			mask: instance.midi_filter.notes
		});
		await getApi().renderNodeUpdateMidiFilterNotes(id, mask);
	}

	function toDb(value) {
		return 10 * Math.log10(value);
	}

	function toLin(value) {
		return Math.pow(10, value / 10);
	}
</script>

<div class="contents">
	<div class="flex flex-row items-start gap-1">
		<div class="w-10 text-center">
			<InputText value={id} readonly={true} class="rounded-l-full w-full" />
		</div>
		<div class="grow">
			<InputText
				on:change={(e) => changeName(e.target.value)}
				value={instance.name}
				class="rounded-r-full" />
		</div>
	</div>
	<BoolProp name={'Enabled'} value={instance.enabled} on:change={(e) => changeEnabled(e.detail)} />
	<BoolProp
		name={'Global transposition'}
		value={!instance.ignore_global_transposition}
		on:change={(e) => changeGlobalTransposition(e.detail)} />
	<NumProp
		name={'Gain'}
		value={toDb(instance.gain)}
		smallStep={1}
		largeStep={5}
		defaultValue={0}
		numDecimalPlaces={1}
		unit={'dB'}
		on:change={(e) => changeGain(toLin(e.detail))} />
	<NumProp
		name={'Transposition'}
		value={instance.transposition}
		smallStep={1}
		largeStep={12}
		defaultValue={0}
		min={-128}
		max={127}
		numDecimalPlaces={0}
		unit={'semit.'}
		on:change={(e) => changeTransposition(e.detail)} />
	<VelocityMapping
		name={'Velocity Mapping'}
		value={instance.velocity_mapping}
		on:change={(e) => changeVelocityMapping(e.detail)} />
	<FileProp
		name={'SFZ File'}
		value={instance.loaded_file}
		on:change={(e) => loadFile(e.detail)}
		allowedExtensions={['sfz']} />
	<FilterChannels
		name="MIDI Channels"
		channels={instance.midi_filter.channels}
		on:change={(e) => updateMidiFilterChannel(e.detail.id, e.detail.value)} />
	<BoolProp
		name={'Sustain Pedal'}
		value={instance.midi_filter.control_commands[64]}
		on:change={(e) => updateMidiFilterSustain(e.detail)} />
	<FilterChannels
		name="User Presets"
		channels={instance.user_presets}
		on:change={(e) => setUserPresetEnabled(e.detail.id, e.detail.value)} />

	<div class="flex flex-row gap-4">
		<div class="grow overflow-hidden text-ellipsis text-nowrap">Keyboard Mask</div>
		<div>
			<!-- <input
				type="checkbox"
				bind:checked={value}
				on:change={(e) => dispatch('change', e.target.checked)}
				class="scale-150" /> -->
			<button
				on:click={openNoteMaskEditor}
				class="grid place-items-center rounded-full border border-solid border-slate-700 bg-slate-800 px-2 py-1 text-slate-300">
				Modify
			</button>
		</div>
	</div>
</div>
