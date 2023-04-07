export let ws: WebSocket = undefined;
let sendQueue = [];
let messageHandlers = new Map();
let allMessageHandler: CallableFunction = console.log;

reconnect();

const websocketEndpoint = location.hostname == 'localhost' ?
    'ws://localhost:4052' :
    `ws://${location.host}`;

export function reconnect(): WebSocket {
    ws = new WebSocket(websocketEndpoint);

    ws.onopen = () => {
        for (let item of sendQueue) {
            ws.send(item);
        }
    }

    ws.onmessage = (e) => {
        allMessageHandler(e.data);
        if (messageHandlers.has(e.data.type)) {
            messageHandlers.get(e.data.type)(JSON.parse(e.data));
        }
    }

    return ws;
}

export function sendMessage(msg: string) {
    if (ws.readyState !== ws.OPEN) {
        sendQueue.push(msg);
        return;
    }
    ws.send(msg);
}

export function setMessageHandler(type: string, callback: (data: object) => void) {
    messageHandlers.set(type, callback);
}

export function setAllMessageHandler(callback: (data: string) => void) {
    console.log(callback)
    allMessageHandler = callback;
}
