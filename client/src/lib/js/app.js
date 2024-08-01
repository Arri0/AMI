import { writable, get } from 'svelte/store';
import { Api } from './api.js';

let api = null;

export const availableMidiInputs = writable([]);
export const connectedMidiInputs = writable([]);
export const cache = writable({ render_nodes: [] });
export const beatState = writable({ beat: 0, div: 0 });

window.getApi = function() {
    return api;
}

window.getCache = function() {
    return get(cache);
}

window.getBeatState = function() {
    return get(beatState);
}

window.addPiano = async function() {
    const id = get(cache).render_nodes.length;
    const api = getApi();
    await api.addRenderNode('SfizzSynth');
    await api.renderNodeSetName(id, 'Piano');
    await api.renderNodeLoadFile(id, 'samples:/Basic Piano.dsbundle/Basic Piano.sfz');
    for(let i = 0; i < 16; ++i) {
        await api.renderNodeUpdateMidiFilterChannel(id, i, i != 9);
    }
}

window.addDrums = async function() {
    const id = get(cache).render_nodes.length;
    const api = getApi();
    await api.addRenderNode('OxiSynth');
    await api.renderNodeSetName(id, 'Drums');
    await api.renderNodeLoadFile(id, 'samples:/Jnsgm2.sf2');
    await api.renderNodeSetBankAndPreset(id, 128, 0);
    for(let i = 0; i < 16; ++i) {
        await api.renderNodeUpdateMidiFilterChannel(id, i, i == 9);
    }
}

window.addDrumMachine = async function() {
    const api = getApi();
    await api.addControlNode('DrumMachine');
}

window.initDefaultSetup = async function() {
    await window.addDrums();
    await window.addPiano();
    await window.addDrumMachine();
    const api = getApi();
    await api.controllerSetEnabled(true);
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

    api.addEventListener('beat-state', (ev) => {
        beatState.set(ev.detail);
    })
}

export function appDestroy() {
    if(api !== null) {
        api.destroy();
        api = null;
    }
}