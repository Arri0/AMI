<script>
	import { getApi, availableMidiInputs, connectedMidiInputs } from '$lib/js/app.js';
	import Icon from '@iconify/svelte';
	import Header from './comp/Header.svelte';
	import Select from '../form/Select.svelte'
	import Section from './comp/Section.svelte';
	import Content from './comp/Content.svelte';

    async function updateSlot(slot, port) {
        const api = getApi();
        if(port.length > 0) {
            await api.connectMidiInput(slot, port);
        } else {
            await api.disconnectMidiInput(slot);
        }
    }
</script>

<Section>
	<Header>
		<span class="inline-block align-middle mx-2">MIDI Ports</span>
		<Icon icon="mdi:midi-port" class="inline-block align-middle" />
	</Header>
	<Content>
		<div class="grid-cols-auto-1fr mx-auto grid max-w-[30rem] items-center gap-2 my-4">
			{#each $connectedMidiInputs as conName, i}
				<div class="inline-block text-nowrap text-slate-300">Slot {i + 1}</div>
				<div class="inline-block">
					<Select options={[['', '--- None ---'], ...$availableMidiInputs]} value={conName ?? ''} on:change={e => updateSlot(i, e.target.value)} />
				</div>
			{/each}
		</div>
	</Content>
</Section>
