<script>
	import { cache } from '$lib/js/app.js';
	import Icon from '@iconify/svelte';
	import Header from '../comp/Header.svelte';
	import Content from '../comp/Content.svelte';
	import RustySynth from './RustySynth.svelte';

	export let currentNode;

	$: kind = $cache.nodes[currentNode]?.kind;
	$: instance = $cache.nodes[currentNode]?.instance;

	function goBackToInstrumentList() {
		currentNode = null;
	}
</script>

<div class="text-slate-300">
	<Header>
		<div class="flex flex-row">
			<div class="w-8">
				<button on:click={goBackToInstrumentList}
					><Icon icon="icon-park-solid:back" class="inline-block align-middle" /></button>
			</div>
			<div class="grow">
				<span class="mx-2 inline-block align-middle">Modify Instrument</span>
				<Icon icon="mage:edit-fill" class="inline-block align-middle" />
			</div>
			<div class="w-8"></div>
		</div>
	</Header>
	<Content>
        <div class="mx-auto my-4 flex max-w-[30rem] select-none flex-col gap-8 px-2">
            {#if kind === 'RustySynth' || kind === 'OxiSynth' || kind === 'FluidliteSynth'}
                <RustySynth id={currentNode} bind:instance />
            {/if}
        </div>
    </Content>
</div>
