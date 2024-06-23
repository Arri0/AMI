export class Api extends EventTarget {
    constructor(host, port) {
        super();
        this.host = host;
        this.port = port;
        this.connect();
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

    async nodeUpdateMidiFilterNote(id, noteId, enabled) {
        return await this.nodeUpdateMidiFilter(id, {
            'Note': [noteId, enabled]
        });
    }

    async nodeUpdateMidiFilterControlChange(id, ccId, enabled) {
        return await this.nodeUpdateMidiFilter(id, {
            'ControlChange': [ccId, enabled]
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
        // this.dispatchEvent(new CustomEvent('broadcast', {
        //     detail: msg.payload
        // }));

        if ('MidiEvent' in msg) {
            this.dispatchEvent(new CustomEvent('midi', {
                detail: msg.MidiEvent
            }));
        } else if ('AvailableMidiInputs' in msg) {
            this.dispatchEvent(new CustomEvent('available-midi-inputs', {
                detail: msg.AvailableMidiInputs
            }));
        } else if ('ConnectedMidiInputs' in msg) {
            this.dispatchEvent(new CustomEvent('connected-midi-inputs', {
                detail: msg.ConnectedMidiInputs
            }));
        } else if ('Cache' in msg) {
            this.dispatchEvent(new CustomEvent('cache', {
                detail: msg.Cache
            }));
        } else if ('RendererResponse' in msg) {
            this.dispatchEvent(new CustomEvent('cache-update', {
                detail: msg.RendererResponse
            }));
        }
    }
}