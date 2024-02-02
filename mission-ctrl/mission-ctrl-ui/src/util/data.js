
export const RAD_TO_DEG = 180.0 / Math.PI;

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