
export const RAD_TO_DEG = 180.0 / Math.PI;

export class DataHandler {
    // websocket_urls: list of websocket URLs to connect to
    // data: initial data to store, intended to be a vue-reactive object
    constructor(websocket_url, data, data_retention_s = 30) {
        this.websocket_url = websocket_url;
        this.websocket = null;
        this.data = data;
        this.data_retention_s = data_retention_s;
        this.last_update = new Date();

        this.data._last_update_elapsed = Infinity;

        this._connect();
        this._heartbeat();
    }

    _heartbeat() {
        if (this.websocket == null || this.websocket.readyState == WebSocket.CLOSED) {
            console.log("Connection lost, reconnecting to websocket");
            this._connect();
        } else if (this.last_update < (new Date() - 5000)) {
            console.log("No data received in 5 seconds, reconnecting to websocket");
            this.websocket.close();
            this._connect();
        }

        this._cullOldData();
        this.data._last_update_elapsed = (new Date() - this.last_update) / 1000;

        setTimeout(this._heartbeat.bind(this), 1000);
    }

    _cullOldData() {
        let now = new Date();

        for (let group_name in this.data) {
            for (let field_name in this.data[group_name]) {
                if (!Array.isArray(this.data[group_name][field_name])) {
                    continue;
                }

                this.data[group_name][field_name] = this.data[group_name][field_name].filter((elem) => {
                    return (now - elem.timestamp) < (this.data_retention_s * 1000.0);
                });
            }
        }
    }

    _connect() {
        let ws = new WebSocket(this.websocket_url);
        ws.onmessage = this._onmessage.bind(this);
        ws.onerror = () => {
            ws.close();
        };
        this.websocket = ws;
    }

    _onmessage(event) {
        this.last_update = new Date();

        let json_data = JSON.parse(event.data);

        let now = new Date();

        // Add new data to history arrays
        for (let group_name in json_data) {
            if (group_name == 'noHistoryFields') {
                continue;
            } else if (json_data['noHistoryFields'].includes(group_name)) {
                this.data[group_name] = json_data[group_name];
                continue;
            }

            if (!(group_name in this.data)) {
                this.data[group_name] = {};
            }

            let group_data = json_data[group_name];
            for (let field_name in group_data) {
                let field_value = group_data[field_name];
                if (!(field_name in this.data[group_name])) {
                    this.data[group_name][field_name] = [];
                }

                this.data[group_name][field_name].push({
                    value: field_value,
                    timestamp: now
                });
            }
        }
    }
}

export class DataFetcher {
    constructor(timeout) {
        this.timeout = timeout;
        this.active_fetching = {};
        this.fetched_data = {};
    }

    // async fetch(url) {
    //     if (!(url in this.active_fetching)) {
    //         this.active_fetching[url] = 0;
    //     }

    //     if (this.active_fetching[url] > 4) {
    //         throw new Error("Too many fetches for " + url);
    //     }

    //     this.active_fetching[url] += 1;

    //     const controller = new AbortController();
    //     const timeoutId = setTimeout(() => {
    //         controller.abort()
    //     }, this.timeout);

    //     const response = await fetch(url);

    //     clearTimeout(timeoutId);
    //     const data = await response.json();

    //     this.fetched_data[url] = data;
    //     this.active_fetching[url] -= 1;
    //     return data;
    // }

    async fetch (url) {
        return await timeoutFetch(url, this.timeout);
    }
}

export async function timeoutFetch(url, timeout) {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => {
        controller.abort()
    }, timeout);

    const response = await fetch(url);

    clearTimeout(timeoutId);
    const data = await response.json();
    return data;
}

export function nvalue(value, defaultValue = 0) {
    if (value != null && value != undefined) {
        return value;
    } else {
        return defaultValue;
    }
}

export function nelem(array, index, defaultValue = null) {
    if (array != null && array != undefined && array.length > index) {
        return array[index];
    } else {
        return defaultValue;
    }
}

export function nmagnitude(array, defaultValue = 0) {
    if (array != null && array != undefined) {
        let sum = 0.0;
        for (let i = 0; i < array.length; i++) {
            sum += array[i] * array[i];
        }
        return Math.sqrt(sum);
    } else {
        return defaultValue;
    }
}

export function nvecstr(array, decimals, defaultValue = "( ?? )") {
    if (array != null && array != undefined) {
        let str = "(";
        for (let i = 0; i < array.length; i++) {
            if (i > 0) {
                str += ", ";
            }
            str += array[i].toFixed(decimals);
        }
        return str + ")";
    } else {
        return defaultValue;
    }
}