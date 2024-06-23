<script>
	import NodeList from './NodeList.svelte';
	import Node from './nodes/Node.svelte';

	export let currentNode;
</script>

{#if currentNode === null}
	<NodeList bind:currentNode />
{:else}
	<Node bind:currentNode />
{/if}