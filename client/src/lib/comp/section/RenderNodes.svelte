<script>
	import RenderNodeList from './RenderNodeList.svelte';
	import Node from './render_nodes/Node.svelte';

	export let currentNode;
</script>

{#if currentNode === null}
	<RenderNodeList bind:currentNode />
{:else}
	<Node bind:currentNode />
{/if}