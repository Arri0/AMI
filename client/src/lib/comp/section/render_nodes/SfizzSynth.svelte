<script>
	import { getApi, openKeyboardEditor } from '$lib/js/app.js';
	import InputText from '../../form/InputText.svelte';
	import BoolProp from './comp/BoolProp.svelte';
	import FileProp from './comp/FileProp.svelte';
	import FilterChannels from './comp/FilterChannels.svelte';
	import NumProp from './comp/NumProp.svelte';
	import VelocityMapping from './comp/VelocityMapping.svelte';

	export let id;
	export let instance;

	async function changeName(newName) {
		await getApi().nodeSetName(id, newName);
	}

	async function changeEnabled(newEnabled) {
		await getApi().nodeSetEnabled(id, newEnabled);
	}

	async function changeGlobalTransposition(newEnabled) {
		await getApi().nodeSetIgnoreGlobalTransposition(id, !newEnabled);
	}

	async function changeGain(newGain) {
		await getApi().nodeSetGain(id, newGain);
	}

	async function changeTransposition(newTransposition) {
		await getApi().nodeSetTransposition(id, newTransposition);
	}

	async function changeVelocityMapping(newMapping) {
		await getApi().nodeSetVelocityMapping(id, newMapping);
	}

	async function loadFile(newFile) {
		await getApi().nodeLoadFile(id, newFile);
	}

	async function updateMidiFilterChannel(channelId, enabled) {
		await getApi().nodeUpdateMidiFilterChannel(id, channelId, enabled);
	}

	async function updateMidiFilterSustain(enabled) {
		await getApi().nodeUpdateMidiFilterControlChange(id, 64, enabled);
	}

	async function openNoteMaskEditor() {
		const mask = await openKeyboardEditor({
			title: 'Keyboard Mask',
			mask: instance.midi_filter.notes
		});
		await getApi().nodeUpdateMidiFilterNotes(id, mask);
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
		<!-- <p class="grow-0"><InputText value={id} readonly={true} /></p>
        <p class="grow"><InputText bind:value={name} /></p> -->
		<div class="w-10 text-center"><InputText value={id} readonly={true} rounded="left" /></div>
		<div class="grow">
			<InputText
				on:change={(e) => changeName(e.target.value)}
				value={instance.name}
				rounded="right" />
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
