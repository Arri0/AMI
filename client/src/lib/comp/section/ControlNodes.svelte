<script>
	import ControlNodeList from './ControlNodeList.svelte';
	import Node from './control_nodes/Node.svelte';

	export let currentNode;
</script>

{#if currentNode === null}
	<ControlNodeList bind:currentNode />
{:else}
	<Node bind:currentNode />
{/if}