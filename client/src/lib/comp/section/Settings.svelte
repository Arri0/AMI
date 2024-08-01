<script>
	import Icon from '@iconify/svelte';
	import Header from './comp/Header.svelte';
	import Section from './comp/Section.svelte';
	import Content from './comp/Content.svelte';
	import MidiPorts from './MidiPorts.svelte';
	import Button from '../form/Button.svelte';

	export let subsection = null;
</script>

{#if subsection === null}
<Section>
	<Header>
		<span class="inline-block align-middle mx-2">Settings</span>
		<Icon icon="material-symbols:settings" class="inline-block align-middle" />
	</Header>
	<Content>
	   <Button on:click={() => (subsection = 'midi-ports')}>Midi Ports</Button>
	</Content>
</Section>
{:else if subsection === 'midi-ports'}
    <MidiPorts on:go-back={() => (subsection = null)} />
{/if}
