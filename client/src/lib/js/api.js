export class Api extends EventTarget {
    constructor(host, port) {
        super();
        this.host = host;
        this.port = port;
        this.connectedMidiInputs = [];
        this.availableMidiInputs = [];
        this.cache = {
            render_nodes: [],
            control_nodes: [],
            controller: {},
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

    async rendererSetUserPreset(presetId) {
        return await this.rendererRequest({
            'SetUserPreset': presetId
        });
    }

    async addRenderNode(kind) {
        return await this.rendererRequest({
            'AddNode': { kind }
        });
    }

    async removeRenderNode(id) {
        return await this.rendererRequest({
            'RemoveNode': { id }
        });
    }

    async cloneRenderNode(id) {
        return await this.rendererRequest({
            'CloneNode': { id }
        });
    }

    async renderNodeRequest(id, kind, timeout) {
        return await this.rendererRequest({
            'NodeRequest': { id, kind }
        }, timeout);
    }

    async renderNodeSetName(id, value) {
        return await this.renderNodeRequest(id, {
            'SetName': value
        });
    }

    async renderNodeSetEnabled(id, value = true) {
        return await this.renderNodeRequest(id, {
            'SetEnabled': value
        });
    }

    async renderNodeLoadFile(id, value) {
        return await this.renderNodeRequest(id, {
            'LoadFile': value
        }, 30000);
    }

    async renderNodeSetGain(id, value) {
        return await this.renderNodeRequest(id, {
            'SetGain': value
        });
    }

    async renderNodeSetTransposition(id, value) {
        return await this.renderNodeRequest(id, {
            'SetTransposition': value
        });
    }

    async renderNodeSetVelocityMapping(id, value) {
        return await this.renderNodeRequest(id, {
            'SetVelocityMapping': value
        });
    }

    async renderNodeSetVelocityMappingIdentity(id) {
        return await this.renderNodeSetVelocityMapping(id, 'Identity');
    }

    async renderNodeSetVelocityMappingLinear(id, min, max) {
        return await this.renderNodeSetVelocityMapping(id, {
            'Linear': {
                min, max
            }
        });
    }

    async renderNodeSetIgnoreGlobalTransposition(id, value=true) {
        return await this.renderNodeRequest(id, {
            'SetIgnoreGlobalTransposition': value
        });
    }

    async renderNodeSetBankAndPreset(id, bank, preset) {
        return await this.renderNodeRequest(id, {
            'SetBankAndPreset': [bank, preset]
        });
    }

    async renderNodeSetSfReverbActive(id, value=true) {
        return await this.renderNodeRequest(id, {
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
    async renderNodeSetSfReverbParams(id, params=true) {
        return await this.renderNodeRequest(id, {
            'SetSfReverbParams': params
        });
    }

    async renderNodeUpdateMidiFilter(id, update) {
        return await this.renderNodeRequest(id, {
            'UpdateMidiFilter': update
        });
    }

    async renderNodeUpdateMidiFilterEnabled(id, enabled=true) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'Enabled': enabled
        });
    }

    async renderNodeUpdateMidiFilterChannel(id, channelId, enabled) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'Channel': [channelId, enabled]
        });
    }

    async renderNodeUpdateMidiFilterChannels(id, channels) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'Channels': channels
        });
    }

    async renderNodeUpdateMidiFilterNote(id, noteId, enabled) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'Note': [noteId, enabled]
        });
    }

    async renderNodeUpdateMidiFilterNotes(id, notes) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'Notes': notes
        });
    }

    async renderNodeUpdateMidiFilterControlChange(id, ccId, enabled) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'ControlChange': [ccId, enabled]
        });
    }

    async renderNodeUpdateMidiFilterControlChanges(id, ccs) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'ControlChanges': ccs
        });
    }

    async renderNodeUpdateMidiFilterProgramChange(id, enabled) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'ProgramChange': enabled
        });
    }

    async renderNodeUpdateMidiFilterChannelAftertouch(id, enabled) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'ChannelAftertouch': enabled
        });
    }

    async renderNodeUpdateMidiFilterPitchWheel(id, enabled) {
        return await this.renderNodeUpdateMidiFilter(id, {
            'PitchWheel': enabled
        });
    }

    async renderNodeSetUserPresetEnabled(id, presetId, enabled=true) {
        return await this.renderNodeRequest(id, {
            'SetUserPresetEnabled': [presetId, enabled]
        });
    }

    async controllerRequest(req, timeout) {
        return await this.request({
            'ControllerRequest': req
        }, timeout);
    }

    async controllerReset() {
        return await this.controllerRequest('Reset');
    }

    async controllerSetEnabled(value) {
        return await this.controllerRequest({
            'SetEnabled': value
        });
    }

    async controllerSetTempoBpm(value) {
        return await this.controllerRequest({
            'SetTempoBpm': value
        });
    }

    async controllerSetRhythm(numBeats, numDivs) {
        console.log('set rhythm:', numBeats, numDivs);
        return await this.controllerRequest({
            'SetRhythm': {
                num_beats: numBeats,
                num_divs: numDivs,
            }
        });
    }

    async controllerSetUserPreset(presetId) {
        return await this.controllerRequest({
            'SetUserPreset': presetId
        });
    }

    async addControlNode(kind) {
        return await this.controllerRequest({
            'AddNode': { kind }
        });
    }

    async removeControlNode(id) {
        return await this.controllerRequest({
            'RemoveNode': { id }
        });
    }

    async cloneControlNode(id) {
        return await this.controllerRequest({
            'CloneNode': { id }
        });
    }

    async controlNodeRequest(id, kind, timeout) {
        return await this.controllerRequest({
            'NodeRequest': { id, kind }
        }, timeout);
    }

    async controlNodeSetName(id, value) {
        return await this.controlNodeRequest(id, {
            'SetName': value
        });
    }

    async controlNodeSetEnabled(id, value = true) {
        return await this.controlNodeRequest(id, {
            'SetEnabled': value
        });
    }

    async controlNodeSavePreset(id, path) {
        return await this.controlNodeRequest(id, {
            'SavePreset': path
        });
    }

    async controlNodeLoadPreset(id, path) {
        return await this.controlNodeRequest(id, {
            'LoadPreset': path
        });
    }

    async controlNodeSetUserPresetEnabled(id, presetId, enabled=true) {
        return await this.controlNodeRequest(id, {
            'SetUserPresetEnabled': [presetId, enabled]
        });
    }

    async controlNodeAddVoice(id) {
        return await this.controlNodeRequest(id, 'AddVoice');
    }

    async controlNodeRemoveVoice(id, voiceId) {
        return await this.controlNodeRequest(id, {
            'RemoveVoice': voiceId
        });
    }

    async controlNodeClearVoices(id) {
        return await this.controlNodeRequest(id, 'ClearVoices');
    }

    async controlNodeSetVoiceName(id, voiceId, name) {
        return await this.controlNodeRequest(id, {
            'SetVoiceName': [voiceId, name]
        });
    }

    async controlNodeSetVoiceInstrument(id, voiceId, instrumentId) {
        return await this.controlNodeRequest(id, {
            'SetVoiceInstrument': [voiceId, instrumentId]
        });
    }

    async controlNodeSetVoiceNote(id, voiceId, note) {
        return await this.controlNodeRequest(id, {
            'SetVoiceNote': [voiceId, note]
        });
    }

    async controlNodeSetVoiceVelocity(id, voiceId, velocity) {
        return await this.controlNodeRequest(id, {
            'SetVoiceVelocity': [voiceId, velocity]
        });
    }

    async controlNodeSetVoiceChannel(id, voiceId, channel) {
        return await this.controlNodeRequest(id, {
            'SetVoiceChannel': [voiceId, channel]
        });
    }

    async controlNodeSetSlot(id, voiceId, slotId, active) {
        return await this.controlNodeRequest(id, {
            'SetSlot': [voiceId, slotId, active]
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
        } else if ('ControllerUpdate' in msg) {
            this._onControllerUpdate(msg.ControllerUpdate);
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

    _onControllerUpdate(update) {
        if('Enabled' in update) {
            this._onControllerEnabledChange(update.Enabled);
        } else if('TempoBpm' in update) {
            this._onControllerTempoBpmChange(update.TempoBpm);
        } else if('Rhythm' in update) {
            this._onControllerRhythmChange(update.Rhythm);
        } else if('BeatState' in update) {
            this._onControllerBeatStateChange(update.BeatState);
        } else if('NodeUpdates' in update) {
            const nodeId = update.NodeUpdates.id;
            const updates = update.NodeUpdates.updates;
            this._onControlNodeUpdates(nodeId, updates);
        } else if('AddNode' in update) {
            this._onAddControlNode(update.AddNode);
        } else if('RemoveNode' in update) {
            this._onRemoveControlNode(update.RemoveNode.id);
        } else if('CloneNode' in update) {
            this._onCloneControlNode(update.CloneNode.id);
        }
    }

    _onControllerEnabledChange(enabled) {
        this.cache.controller.enabled = enabled;
        this._dispatchCacheUpdate();
    }

    _onControllerTempoBpmChange(tempoBpm) {
        this.cache.controller.tempo_bpm = tempoBpm;
        this._dispatchCacheUpdate();
    }

    _onControllerRhythmChange(rhythm) {
        this.cache.controller.rhythm = rhythm;
        this._dispatchCacheUpdate();
    }

    _onControllerBeatStateChange(beatState) {
        this.dispatchEvent(new CustomEvent('beat-state', {
            detail: beatState
        }));
    }

    _onControlNodeUpdates(id, updates) {
        const instance = this.cache.control_nodes[id].instance;
        for(const [key, value] of updates) {
            instance[key] = value;
        }
        this._dispatchCacheUpdate();
    }

    _onAddControlNode(node) {
        this.cache.control_nodes.push(node);
        this._dispatchCacheUpdate();
    }

    _onRemoveControlNode(id) {
        this.cache.control_nodes.splice(id, 1);
        this._dispatchCacheUpdate();
    }

    _onCloneControlNode(id) {
        const node = JSON.parse(JSON.stringify(this.cache.control_nodes[id]));
        this.cache.control_nodes.push(node);
        this._dispatchCacheUpdate();
    }

    _dispatchCacheUpdate() {
        this.dispatchEvent(new CustomEvent('cache-update', {
            detail: this.cache
        }));
    }
}