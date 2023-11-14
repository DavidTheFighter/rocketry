
export async function timeoutFetch(url, timeout) {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => {
        controller.abort()
    }, timeout)

    const response = await fetch(url, {
        signal: controller.signal,
    })

    clearTimeout(timeoutId)
    const data = await response.json()
    return data
};

