addToLibrary({
    js_fetch: async function (urlPtr) {
        const url = UTF8ToString(urlPtr);

        const res = await fetch(url);
        if (!res.ok) {
            console.error("fetch failed", res.status, url);
            return 0;
        }

        const bytes = new Uint8Array(await res.arrayBuffer());
        const bytesStructPtr = _alloc(bytes.length);

        const dataPtr = getValue(bytesStructPtr + 4, "*");
        HEAPU8.set(bytes, dataPtr);

        return bytesStructPtr;
    },
});
