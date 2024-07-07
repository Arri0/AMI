export class Api extends EventTarget {
    constructor(host, port) {
        super();
        this.host = host;
        this.port = port;
        this.connectedMidiInputs = [];
        this.availableMidiInputs = [];
        this.cache = {
            render_nodes: [],
            drum_machine: {},
        };
        this.connect();
    }

    getCache() {
        return this.cache;
    }

    getRenderNodes() {
        return this.cache.render_nodes;
    }

    getDrumMachine() {
        return this.cache.drum_machine;
    }

    connect() {
        this.socket = new WebSocket(`ws://${this.host}:${this.port}/ws`);
        this.idCounter = 0;
        this.requestCallbacks = {};

        this.socket.addEventListener('open', () => {
            this.dispatchEvent(new CustomEvent('connected'));
        });

        this.socket.addEventListener('message', (event) => {
            try {
                const msg = JSON.parse(event.data);

                if (msg.id === 0 && msg.response === false) {
                    this._onBroadcast(msg.payload);
                } else if (msg.response === true) {
                    this._resolveRequest(msg.id, msg.payload);
                }
            } catch (e) {
                console.error("Invalid message received:", event.data);
                console.error(e);
            }
        });

        this.socket.addEventListener('close', () => {
            this.dispatchEvent(new CustomEvent('disconnected'));
            setTimeout(() => (this.connect()), 1000);
        });
    }

    destroy() {
        this.socket.close();
    }

    send(msg, request = false) {
        this.socket.send(JSON.stringify({
            id: ++this.idCounter,
            request,
            payload: msg,
        }));
        return this.idCounter;
    }

    async request(msg, timeout = 1000) {
        const id = this.send(msg, true);
        return new Promise((resolve, reject) => {
            this._addRequest(id, resolve, reject);
            this._setupRequestTimeout(id, timeout);
        })
    }

    async connectMidiInput(slot, inputName) {
        return await this.request({
            'ConnectMidiInput': [slot, inputName]
        })
    }

    async disconnectMidiInput(slot) {
        return await this.request({
            'DisconnectMidiInput': slot
        })
    }

    async rendererRequest(req, timeout) {
        return await this.request({
            'RendererRequest': req
        }, timeout);
    }

    async addNode(kind) {
        return await this.rendererRequest({
            'AddNode': { kind }
        });
    }

    async removeNode(id) {
        return await this.rendererRequest({
            'RemoveNode': { id }
        });
    }

    async cloneNode(id) {
        return await this.rendererRequest({
            'CloneNode': { id }
        });
    }

    async nodeRequest(id, kind, timeout) {
        return await this.rendererRequest({
            'NodeRequest': { id, kind }
        }, timeout);
    }

    async nodeSetName(id, value) {
        return await this.nodeRequest(id, {
            'SetName': value
        });
    }

    async nodeSetEnabled(id, value = true) {
        return await this.nodeRequest(id, {
            'SetEnabled': value
        });
    }

    async nodeLoadFile(id, value) {
        return await this.nodeRequest(id, {
            'LoadFile': value
        }, 30000);
    }

    async nodeSetGain(id, value) {
        return await this.nodeRequest(id, {
            'SetGain': value
        });
    }

    async nodeSetTransposition(id, value) {
        return await this.nodeRequest(id, {
            'SetTransposition': value
        });
    }

    async nodeSetVelocityMapping(id, value) {
        return await this.nodeRequest(id, {
            'SetVelocityMapping': value
        });
    }

    async nodeSetVelocityMappingIdentity(id) {
        return await this.nodeSetVelocityMapping(id, 'Identity');
    }

    async nodeSetVelocityMappingLinear(id, min, max) {
        return await this.nodeSetVelocityMapping(id, {
            'Linear': {
                min, max
            }
        });
    }

    async nodeSetIgnoreGlobalTransposition(id, value=true) {
        return await this.nodeRequest(id, {
            'SetIgnoreGlobalTransposition': value
        });
    }

    async nodeSetBankAndPreset(id, bank, preset) {
        return await this.nodeRequest(id, {
            'SetBankAndPreset': [bank, preset]
        });
    }

    async nodeSetSfReverbActive(id, value=true) {
        return await this.nodeRequest(id, {
            'SetSfReverbActive': value
        });
    }

    /**
     * params: {
     *   room_size: f32,
     *   damping: f32,
     *   width: f32,
     *   level: f32,
     * }
     *  */
    async nodeSetSfReverbParams(id, params=true) {
        return await this.nodeRequest(id, {
            'SetSfReverbParams': params
        });
    }

    async nodeUpdateMidiFilter(id, update) {
        return await this.nodeRequest(id, {
            'UpdateMidiFilter': update
        });
    }

    async nodeUpdateMidiFilterEnabled(id, enabled=true) {
        return await this.nodeUpdateMidiFilter(id, {
            'Enabled': enabled
        });
    }

    async nodeUpdateMidiFilterChannel(id, channelId, enabled) {
        return await this.nodeUpdateMidiFilter(id, {
            'Channel': [channelId, enabled]
        });
    }

    async nodeUpdateMidiFilterChannels(id, channels) {
        return await this.nodeUpdateMidiFilter(id, {
            'Channels': channels
        });
    }

    async nodeUpdateMidiFilterNote(id, noteId, enabled) {
        return await this.nodeUpdateMidiFilter(id, {
            'Note': [noteId, enabled]
        });
    }

    async nodeUpdateMidiFilterNotes(id, notes) {
        return await this.nodeUpdateMidiFilter(id, {
            'Notes': notes
        });
    }

    async nodeUpdateMidiFilterControlChange(id, ccId, enabled) {
        return await this.nodeUpdateMidiFilter(id, {
            'ControlChange': [ccId, enabled]
        });
    }

    async nodeUpdateMidiFilterControlChanges(id, ccs) {
        return await this.nodeUpdateMidiFilter(id, {
            'ControlChanges': ccs
        });
    }

    async nodeUpdateMidiFilterProgramChange(id, enabled) {
        return await this.nodeUpdateMidiFilter(id, {
            'ProgramChange': enabled
        });
    }

    async nodeUpdateMidiFilterChannelAftertouch(id, enabled) {
        return await this.nodeUpdateMidiFilter(id, {
            'ChannelAftertouch': enabled
        });
    }

    async nodeUpdateMidiFilterPitchWheel(id, enabled) {
        return await this.nodeUpdateMidiFilter(id, {
            'PitchWheel': enabled
        });
    }

    async nodeSetUserPreset(id, presetId) {
        return await this.nodeRequest(id, {
            'SetUserPreset': presetId
        });
    }

    async nodeSetUserPresetEnabled(id, presetId, enabled=true) {
        return await this.nodeRequest(id, {
            'SetUserPresetEnabled': [presetId, enabled]
        });
    }

    async readDir(path) {
        const res = (await this.request({
            'ReadDir': path
        })).DirInfo;

        if(!Array.isArray(res))
            return null;

        const dirs = [];
        const files = [];

        for(const rec of res) {
            if(rec[0])
                dirs.push(rec[1]);
            else
                files.push(rec[1]);
        }

        return [dirs.sort(), files.sort()]
    }

    async drumMachineRequest(kind, timeout) {
        return await this.request({
            'DrumMachineRequest': kind
        }, timeout);
    }

    async drumMachineSetEnabled(value=true) {
        return await this.drumMachineRequest({
            'SetEnabled': value
        });
    }

    async drumMachineAddVoice() {
        return await this.drumMachineRequest('AddVoice');
    }

    async drumMachineRemoveVoice(voiceId) {
        return await this.drumMachineRequest({
            'RemoveVoice': voiceId
        });
    }

    async drumMachineClearVoices() {
        return await this.drumMachineRequest('ClearVoices');
    }

    async drumMachineSetVoiceName(voiceId, name) {
        return await this.drumMachineRequest({
            'SetVoiceName': [voiceId, name]
        });
    }

    async drumMachineSetVoiceInstrument(voiceId, instrumentId) {
        return await this.drumMachineRequest({
            'SetVoiceInstrument': [voiceId, instrumentId]
        });
    }

    async drumMachineSetVoiceNote(voiceId, note) {
        return await this.drumMachineRequest({
            'SetVoiceNote': [voiceId, note]
        });
    }

    async drumMachineSetSlot(voiceId, slotId, velocity) {
        return await this.drumMachineRequest({
            'SetSlot': [voiceId, slotId, velocity]
        });
    }

    async drumMachineSetRhythm(numBeats, numDivs) {
        return await this.drumMachineRequest({
            'SetRhythm': {
                num_beats: numBeats,
                num_divs: numDivs,
            }
        });
    }

    async drumMachineSetTempoBpm(tempoBpm) {
        return await this.drumMachineRequest({
            'SetTempoBpm': tempoBpm
        });
    }

    async drumMachineReset() {
        return await this.drumMachineRequest('Reset');
    }

    async drumMachineLoadPreset(path) {
        return await this.drumMachineRequest({
            'LoadPreset': path
        });
    }

    async drumMachineSavePreset(path) {
        return await this.drumMachineRequest({
            'SavePreset': path
        });
    }

    _addRequest(id, resolve, reject) {
        this.requestCallbacks[id] = {
            resolve,
            reject,
        };
    }

    _resolveRequest(id, result) {
        if (id in this.requestCallbacks) {
            const cb = this.requestCallbacks[id];
            clearTimeout(cb.timeout);
            cb.resolve(result);
            delete this.requestCallbacks[id];
        }
    }

    _setupRequestTimeout(id, timeout) {
        this.requestCallbacks[id].timeout = setTimeout(() => {
            if (id in this.requestCallbacks) {
                const cb = this.requestCallbacks[id];
                cb.reject(new Error('timeout'));
                delete this.requestCallbacks[id];
            }
        }, timeout)
    }

    _onBroadcast(msg) {
        if ('MidiEvent' in msg) {
            this.dispatchEvent(new CustomEvent('midi', {
                detail: msg.MidiEvent
            }));
        } else if ('AvailableMidiInputs' in msg) {
            this.availableMidiInputs = msg.AvailableMidiInputs;
            this.dispatchEvent(new CustomEvent('available-midi-inputs', {
                detail: this.availableMidiInputs
            }));
        } else if ('ConnectedMidiInputs' in msg) {
            this.connectedMidiInputs = msg.ConnectedMidiInputs;
            this.dispatchEvent(new CustomEvent('connected-midi-inputs', {
                detail: this.connectedMidiInputs
            }));
        } else if ('Cache' in msg) {
            this._onCacheReceived(msg.Cache);
        } else if ('RendererUpdate' in msg) {
            this._onRendererUpdate(msg.RendererUpdate);
        } else if ('DrumMachineUpdates' in msg) {
            this._onDrumMachineUpdates(msg.DrumMachineUpdates);
        }
    }

    _onCacheReceived(cache) {
        this.cache = cache;
        this.dispatchEvent(new CustomEvent('cache-update', {
            detail: cache
        }));
    }

    _onRendererUpdate(update) {
        if('NodeUpdates' in update) {
            const nodeId = update.NodeUpdates.id;
            const updates = update.NodeUpdates.updates;
            this._onRenderNodeUpdates(nodeId, updates);
        } else if('AddNode' in update) {
            this._onAddRenderNode(update.AddNode);
        } else if('RemoveNode' in update) {
            this._onRemoveRenderNode(update.RemoveNode.id);
        } else if('CloneNode' in update) {
            this._onCloneRenderNode(update.CloneNode.id);
        }

        this.dispatchEvent(new CustomEvent('cache-update', {
            detail: this.cache
        }));
    }

    _onRenderNodeUpdates(id, updates) {
        const instance = this.cache.render_nodes[id].instance;
        for(const [key, value] of updates) {
            instance[key] = value;
        }
    }

    _onAddRenderNode(node) {
        this.cache.render_nodes.push(node);
    }

    _onRemoveRenderNode(id) {
        this.cache.render_nodes.splice(id, 1);
    }

    _onCloneRenderNode(id) {
        const node = JSON.parse(JSON.stringify(this.cache.render_nodes[id]));
        this.cache.render_nodes.push(node);
    }

    _onDrumMachineUpdates(updates) {
        console.log('dm updates', updates)
        const drumMachine = this.cache.drum_machine;
        for(const [key, value] of updates) {
            drumMachine[key] = value;
        }
        this.dispatchEvent(new CustomEvent('cache-update', {
            detail: this.cache
        }));
    }
}