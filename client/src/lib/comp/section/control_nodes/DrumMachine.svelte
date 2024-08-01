<script>
	import { getApi, cache, beatState, openFileBrowser } from '$lib/js/app.js';
	import Icon from '@iconify/svelte';
	import InputText from '../../form/InputText.svelte';
	import BoolProp from '../node_comp/BoolProp.svelte';
	import Button from '../../form/Button.svelte';
	import Select from '../../form/Select.svelte';
	import FilterChannels from '../node_comp/FilterChannels.svelte';

	export let id;
	export let instance;

	const CHANNELS = Array.from({ length: 16 }, (_, i) => [i, `Channel ${i}`]);
	const NOTES = Array.from({ length: 128 }, (_, i) => [i, `Note ${i}`]);

	$: instruments = $cache.render_nodes.map((n, i) => [i, n.instance.name]);
	$: rhythm = $cache.controller.rhythm;
	$: activeSlot = $beatState.beat * rhythm.num_divs + $beatState.div;

	async function changeName(newName) {
		console.log(await getApi().controlNodeSetName(id, newName));
	}

	async function changeEnabled(newEnabled) {
		console.log(await getApi().controlNodeSetEnabled(id, newEnabled));
	}

	async function addVoice() {
		console.log(await getApi().controlNodeAddVoice(id));
	}

	async function removeVoice(voice) {
		console.log(await getApi().controlNodeRemoveVoice(id, voice));
	}

	async function setVoiceName(voice, name) {
		console.log(await getApi().controlNodeSetVoiceName(id, voice, name));
	}

	async function setInstrument(voice, instrumentIndex) {
		instrumentIndex = Number.isInteger(instrumentIndex) ? instrumentIndex : null;
		console.log(await getApi().controlNodeSetVoiceInstrument(id, voice, instrumentIndex));
	}

	async function setChannel(voice, channel) {
		console.log(await getApi().controlNodeSetVoiceChannel(id, voice, channel));
	}

	async function setNote(voice, note) {
		console.log(await getApi().controlNodeSetVoiceNote(id, voice, note));
	}

	async function setSlot(voice, slot, active) {
		console.log(await getApi().controlNodeSetSlot(id, voice, slot, active));
	}

	async function setVelocity(voice, velocity) {
		console.log(await getApi().controlNodeSetVoiceVelocity(id, voice, velocity));
	}

	async function setUserPresetEnabled(presetId, enabled) {
		await getApi().controlNodeSetUserPresetEnabled(id, presetId, enabled);
	}

	async function spawnFileBrowserToLoad() {
		const file = await openFileBrowser({
			title: 'Load Drum Machine Preset',
			buttonText: 'Load',
			path: 'beats:',
			allowedExtensions: ['dmp']
		});
		if (file !== null) {
			console.log(await getApi().controlNodeLoadPreset(id, file));
		}
	}

	async function spawnFileBrowserToSave() {
		const file = await openFileBrowser({
			title: 'Save Drum Machine Preset',
			buttonText: 'Save',
			path: 'beats:',
			allowedExtensions: ['dmp']
		});
		if (file !== null) {
			console.log(await getApi().controlNodeSavePreset(id, file));
		}
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
	<div
		class="scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent box-border select-none overflow-x-auto p-4">
		<div class="mb-4 flex flex-row gap-1 border-b-2 border-slate-800 pb-4">
			<Button class="rounded-l-full" on:click={addVoice}>Add Voice</Button>
			<Button on:click={spawnFileBrowserToLoad}>Load Preset</Button>
			<Button class="rounded-r-full" on:click={spawnFileBrowserToSave}>Save Preset</Button>
		</div>
		<div class="flex flex-col gap-4">
			{#each instance.voices.voices as voice, voiceId}
				<div class="box-border flex flex-row gap-1">
					<div class="w-8">
						<button
							on:click={() => removeVoice(voiceId)}
							class="grid h-8 w-8 place-items-center text-slate-300">
							<Icon icon="subway:delete" />
						</button>
					</div>
					<div class="w-44 flex-shrink-0 text-ellipsis text-nowrap">
						<InputText
							class="box-border h-full w-full rounded-l-full"
							value={voice.name}
							on:change={(e) => setVoiceName(voiceId, e.target.value)} />
					</div>
					<div class="w-44 flex-shrink-0">
						<Select
							class="h-full w-full"
							options={[['', '--- None ---'], ...instruments]}
							on:change={(e) => setInstrument(voiceId, parseInt(e.target.value))}
							value={voice.instrument_index ?? ''} />
					</div>
					<div class="w-28 flex-shrink-0">
						<Select
							class="h-full w-full"
							options={CHANNELS}
							on:change={(e) => setChannel(voiceId, parseInt(e.target.value))}
							value={voice.channel} />
					</div>
					<div class="w-28 flex-shrink-0">
						<Select
							class="h-full w-full rounded-r-full"
							options={NOTES}
							on:change={(e) => setNote(voiceId, parseInt(e.target.value))}
							value={voice.note} />
					</div>
					<div>
						<input
							type="range"
							min="0"
							max="127"
							step="1"
							value={voice.velocity}
							on:change={(e) => setVelocity(voiceId, parseInt(e.target.value))}
							class="align-middle accent-slate-500" />
					</div>
					<div class="flex flex-row">
						{#each voice.slots as slot, slotId}
							{#if slotId !== 0 && slotId % rhythm.num_divs === 0}
								<div class="w-1 flex-shrink-0"></div>
							{/if}
							<button
								on:click={() => setSlot(voiceId, slotId, !slot)}
								class="grid h-6 w-6 place-items-center border border-solid border-slate-700 {activeSlot ===
								slotId
									? 'bg-slate-700'
									: 'bg-slate-800'} text-slate-300">
								<span class:hidden={!slot}><Icon icon="wpf:checkmark" /></span>
							</button>
						{/each}
					</div>
					<div class="w-2 flex-shrink-0"></div>
				</div>
			{/each}
		</div>
	</div>
	<FilterChannels
		name="User Presets"
		channels={instance.user_presets}
		on:change={(e) => setUserPresetEnabled(e.detail.id, e.detail.value)} />
	<!-- <textarea>{JSON.stringify(instance, null, 4)}</textarea> -->
</div>
