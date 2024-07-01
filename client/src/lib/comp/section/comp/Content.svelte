<script>
    export let centerY = false;
</script>

<div class="{centerY ? 'grid place-items-center h-full place-content-center' : ''} overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent">
    <slot></slot>
</div>