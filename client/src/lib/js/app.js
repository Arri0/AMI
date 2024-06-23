import { writable } from 'svelte/store';
import { Api } from './api.js';

let api = null;

export const availableMidiInputs = writable([]);
export const connectedMidiInputs = writable([]);
export const cache = writable({
    nodes: []
});

export let openFileBrowser = () => {};

export function registerFileBrowser(openFn) {
    openFileBrowser = openFn;
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

    api.addEventListener('cache', (ev) => {
        cache.set(ev.detail);
        console.log('cache', ev.detail);
    });

    api.addEventListener('cache-update', (ev) => {
        console.log('cache-update', ev.detail);
        const update = ev.detail;
        if('NodeResponse' in update) {
            const nodeId = update.NodeResponse.id;
            const kind = update.NodeResponse.kind;
            handleNodeResponse(nodeId, kind);
        } else if('AddNode' in update) {
            addNode(update.AddNode.kind, update.AddNode.instance);
        } else if('RemoveNode' in update) {
            removeNode(update.RemoveNode.id);
        } else if('CloneNode' in update) {
            cloneNode(update.CloneNode.id);
        }
    });
}

export function appDestroy() {
    if(api !== null) {
        api.destroy();
        api = null;
    }
}

function addNode(kind, instance) {
    cache.update((value) => {
        value.nodes.push({ kind, instance });
        return value;
    });
}

function removeNode(id) {
    cache.update((value) => {
        value.nodes.splice(id, 1);
        return value;
    });
}

function cloneNode(id) {
    cache.update((value) => {
        const node = JSON.parse(JSON.stringify(value.nodes[id]));
        value.nodes.push(node);
        return value;
    });
}

function handleNodeResponse(id, kind) {
    if(typeof kind === 'object' && !Array.isArray(kind) && kind !== null) {
        if('UpdateFields' in kind) {
            updateNodeFields(id, kind.UpdateFields);
        }
    }
}

function updateNodeFields(id, fields) {
    cache.update((value) => {
        const instance = value.nodes[id].instance;
        for(const field of fields) {
            instance[field[0]] = field[1];
        }
        return value;
    })
}