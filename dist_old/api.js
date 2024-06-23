export class Api extends EventTarget {
    constructor(host, port) {
        super();
        this.socket = new WebSocket(`ws://${host}:${port}/ws`);
        this.idCounter = 0;

        this.socket.addEventListener('open', (event) => {
            this.dispatchEvent(new CustomEvent('connected'));
        });

        this.socket.addEventListener('message', (event) => {
            console.log('Message from server', event.data);
        });

        this.socket.addEventListener('close', () => {
            this.dispatchEvent(new CustomEvent('disconnected'));
        })
    }

    send(msg, request=false) {
        this.socket.send(JSON.stringify({
            id: this.idCounter++,
            request,
            payload: msg,
        }));
    }

    request(msg) {
        this.send(msg, true);
    }
}