<script>
    import { createEventDispatcher } from 'svelte';
    import Icon from '@iconify/svelte';

    export let currentSection = 'nodes';
    export let previousSection = currentSection;

    let dispatch = createEventDispatcher();

    export const sections = [
        ['midi-ports', 'mdi:midi-port'],
        ['nodes', 'game-icons:grand-piano'],
        ['drum-machine', 'lucide:drum'],
        ['log', 'octicon:log-16'],
        ['settings', 'material-symbols:settings'],
    ];

    function setSection(name) {
        previousSection = currentSection;
        currentSection = name;
        dispatch('section-changed');
    }
</script>

<div class="flex flex-row justify-center gap-7 text-2xl py-4 bg-slate-800 text-slate-500">
    {#each sections as [name, icon]}
        <button class:text-slate-300={currentSection===name} on:click={() => setSection(name)}><Icon icon={icon} /></button>
    {/each}
</div>