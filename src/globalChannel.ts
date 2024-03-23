import { UnlistenFn } from "@tauri-apps/api/event";
import { customSchemeLocalhost } from "./customScheme.ts";
import { listen } from "@tauri-apps/api/event";
import { createContext } from "preact";

const localhost = await customSchemeLocalhost("bin-ipc-global-channel");

let unlisten: UnlistenFn;
export const receiver = new ReadableStream({
    type: "bytes",
    async start(controller) {
        unlisten = await listen("bin-ipc-global-channel:ready", async () => {
            const res = await fetch(`${localhost}/pop`, {
                method: "POST",
            });

            switch (res.status) {
                case 100: {
                    break;
                }
                case 200: {
                    const buf = await res.arrayBuffer();
                    controller.enqueue(new Uint8Array(buf));
                    break;
                }
                case 204: {
                    controller.close();
                    break;
                }
            }
        });
    },
    cancel() {
        unlisten();
    },
});

export const sender = new WritableStream<Uint8Array>({
    async write(chunk) {
        await fetch(`${localhost}/push`, {
            method: "POST",
            body: chunk,
        });
    },
});

export const GlobalChannel = createContext({
    receiver,
    sender,
});