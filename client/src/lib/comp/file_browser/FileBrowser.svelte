<script>
    import { getApi, registerFileBrowser } from '$lib/js/app.js';
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import Header from '../section/comp/Header.svelte';
	import PathInspector from './PathInspector.svelte';
	import Inputs from './Inputs.svelte';

    let hidden = true;
    let title = '';
    let dirs = [];
    let files = [];
    let path = null;
    let buttonText = 'Open File';
    let selectedFile = '';
    let allowedExtensions = null;
    let cbResolve = null;

    async function open(params) {
        title = params.title ?? 'Open File';
        path = params.path ?? null;
        allowedExtensions = params.allowedExtensions ?? null;
        buttonText = params.buttonText ?? 'Open';
        if(title === null || path === null)
            return;
        await loadDir();
        hidden = false;
        return new Promise((resolve, _) => {
            cbResolve = resolve;
        });
    }

    async function onDirClicked(dirName) {
        path += `/${dirName}`;
        await loadDir();
    }

    function onFileClicked(fileName) {
        selectedFile = fileName;
    }

    async function loadDir() {
        [dirs, files] = await getApi().readDir(path);
        files = files.filter((fileName) => hasAllowedExtension(fileName));
    }

    function hasAllowedExtension(fileName) {
        if(allowedExtensions === null)
            return true;
        const components = fileName.split('.');
        const ext = components[components.length-1];
        for(const aext of allowedExtensions) {
            if(ext.toLowerCase() === aext.toLowerCase())
                return true;
        }
        return false;
    }

    function close() {
        hidden = true;
        cbResolve(null);
        cbResolve = null;
    }

    async function onPathSelected(newPath) {
        if(path !== newPath) {
            path = newPath;
            await loadDir();
        }
    }

    function onFileSelected(fileName) {
        hidden = true;
        cbResolve(`${path}/${fileName}`);
        cbResolve = null;
    }

    onMount(() => {
        registerFileBrowser(open);
    });
</script>

<div class="fixed inset-0 z-[9999] bg-slate-950 text-slate-300 grid grid-rows-file-browser grid-cols-1" class:hidden>
    <Header>
        <div class="flex flex-row">
            <div class="w-8"></div>
            <div class="grow text-center">
                <span class="inline-block align-middle mx-2">{title}</span>
                <!-- <Icon icon="octicon:file-directory-open-fill-24" class="inline-block align-middle" /> -->
            </div>
            <div class="w-8">
                <button on:click={close}><Icon icon="fa:close" class="inline-block align-middle" /></button>
            </div>
        </div>
    </Header>
    <PathInspector bind:path on:path-selected={(e) => onPathSelected(e.detail)} />
    <div class="text-slate-300 flex flex-col gap-2 p-2 overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent">
        {#each dirs as dir}
            <button on:click={() => onDirClicked(dir)} class="rounded-full bg-slate-900 py-2 px-4 select-none text-left hover:bg-slate-800 transition-colors ease-out">
                <Icon icon="octicon:file-directory-open-fill-24" class="inline-block align-middle" />
                <span class="inline-block align-middle ml-2 select-text">{dir}</span>
            </button>
        {/each}
        {#each files as file}
            <button on:click={() => onFileClicked(file)} class="rounded-full bg-slate-900 py-2 px-4 select-none text-left hover:bg-slate-800 transition-colors ease-out">
                <Icon icon="flowbite:file-solid" class="inline-block align-middle" />
                <span class="inline-block align-middle ml-2 select-text">{file}</span>
            </button>
        {/each}
    </div>
    <Inputs bind:buttonText bind:fileName={selectedFile} on:file-selected={(e) => onFileSelected(e.detail)}  />
</div>