<script>
	import { cache, getApi } from '$lib/js/app.js';
	import Header from './comp/Header.svelte';
	import Icon from '@iconify/svelte';
	import Section from './comp/Section.svelte';
	import Content from './comp/Content.svelte';

	export let currentNode;

	const NODE_KINDS = [
		['RustySynth', 'Rusty Synth'],
		['OxiSynth', 'Oxi Synth'],
		['FluidliteSynth', 'Fluidlite Synth'],
		['SfizzSynth', 'Sfizz Synth'],
	];

	$: nodes = $cache.nodes;

	async function newNode(kind) {
		console.log(await getApi().addNode(kind));
	}

	async function removeNode(id) {
		console.log(await getApi().removeNode(id));
	}

	async function cloneNode(id) {
		console.log(await getApi().cloneNode(id));
	}

	async function toggleNode(id) {
		const prevValue = nodes[id]["instance"].enabled;
		if(await getApi().nodeSetEnabled(id, !prevValue) != 'Ack') {
			console.error('toggleNode: fail');
		}
	}

	function openNode(id) {
		currentNode = id;
	}
</script>

<Section>
	<Header>
		<span class="mx-2 inline-block align-middle">Nodes</span>
		<Icon icon="game-icons:grand-piano" class="inline-block align-middle" />
	</Header>

	<Content>
		<div class="mx-auto my-4 flex max-w-[30rem] select-none flex-col gap-2 px-2">
			{#each nodes as {_kind,instance}, nodeId}
				<div class="flex flex-row items-center gap-4">
					<div class="flex grow flex-row">
						<button on:click={() => toggleNode(nodeId)} class="w-8 rounded-l-full {instance.enabled ? 'bg-green-600' : 'bg-red-700'}"></button>
						<button
							on:click={() => openNode(nodeId)}
							class="grow rounded-r-full bg-slate-800 px-4 py-2 text-left text-sm"
							>{instance.name}</button>
					</div>
					<div class="flex flex-row items-center gap-4 text-xl">
						<!-- <button class="flex items-center"><Icon icon="teenyicons:up-solid" /></button>
						<button class="flex items-center"><Icon icon="teenyicons:down-solid" /></button> -->
						<button on:click={() => cloneNode(nodeId)} class="flex items-center"><Icon icon="ic:sharp-file-copy" /></button>
						<button on:click={() => removeNode(nodeId)} class="flex items-center"
							><Icon icon="f7:bin-xmark-fill" /></button>
					</div>
				</div>
			{/each}
			<div class="mt-4 flex flex-row gap-2 text-xs flex-wrap">
				{#each NODE_KINDS as kind}
					<button
						on:click={() => newNode(kind[0])}
						class="flex flex-row items-center gap-2 rounded-full bg-slate-800 px-4 py-2"
						><Icon icon="mingcute:plus-fill" class="inline" />{kind[1]}</button>
				{/each}
			</div>
		</div>

	</Content>
</Section>
