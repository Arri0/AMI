import { writable, get } from 'svelte/store';
import { Api } from './api.js';

let api = null;

export const availableMidiInputs = writable([]);
export const connectedMidiInputs = writable([]);
export const cache = writable({ render_nodes: [] });

window.getApi = function() {
    return api;
}

window.getCache = function() {
    return get(cache);
}

window.addPiano = async function() {
    const id = get(cache).render_nodes.length;
    console.log('id', id);
    const api = getApi();
    await api.addNode('SfizzSynth');
    await api.nodeLoadFile(id, 'samples:/Basic Piano.dsbundle/Basic Piano.sfz');
    for(let i = 0; i < 16; ++i) {
        await api.nodeUpdateMidiFilterChannel(id, i, i != 9);
    }
}

window.addDrums = async function() {
    const id = get(cache).render_nodes.length;
    console.log('id', id);
    const api = getApi();
    await api.addNode('OxiSynth');
    await api.nodeLoadFile(id, 'samples:/MS_Basic.sf2');
    await api.nodeSetBankAndPreset(id, 128, 25);
    for(let i = 0; i < 16; ++i) {
        await api.nodeUpdateMidiFilterChannel(id, i, i == 9);
    }
}

export let openFileBrowser = async () => {};

export function registerFileBrowser(openFn) {
    openFileBrowser = openFn;
}

export let openKeyboardEditor = async () => {};

export function registerKeyboardEditor(openFn) {
    openKeyboardEditor = openFn;
}

export function getApi() {
    return api;
}

export function appInit() {
    api = new Api(location.hostname, 3000);//TODO: use location.port instead

    api.addEventListener('connected', async () => {
        console.log('Connected');
    });

    api.addEventListener('disconnected', () => {
        console.log('Disconnected');
    });

    api.addEventListener('available-midi-inputs', (ev) => {
        availableMidiInputs.set(ev.detail)
    });

    api.addEventListener('connected-midi-inputs', (ev) => {
        connectedMidiInputs.set(ev.detail);
    });

    api.addEventListener('cache-update', (ev) => {
        cache.set(ev.detail);
        console.log('cache-update', ev.detail);
    });
}

export function appDestroy() {
    if(api !== null) {
        api.destroy();
        api = null;
    }
}