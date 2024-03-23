import { createContext, h, render } from "preact";
import App from "./App.tsx";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { platform } from "@tauri-apps/api/os";

// deno-lint-ignore no-unused-labels
DEV: {
  new EventSource("/esbuild").addEventListener("change", (e) => {
    const { added, removed, updated } = JSON.parse(e.data);

    if (!added.length && !removed.length && updated.length === 1) {
      for (const link of Array.from(document.getElementsByTagName("link"))) {
        const url = new URL(link.href);

        if (url.host === location.host && url.pathname === updated[0]) {
          const next = link.cloneNode() as HTMLLinkElement;
          next.href = updated[0] + "?" + Math.random().toString(36).slice(2);
          next.onload = () => link.remove();
          link.parentNode!.insertBefore(next, link.nextSibling);
          return;
        }
      }
    }

    location.reload();
  });
}

async function customSchemeLocalhost(scheme: string) {
  const name = await platform();
  if (name === "win32") {
    return `https://${scheme}.localhost`;
  }

  return `${scheme}://localhost`;
}

const localhost = await customSchemeLocalhost("bin-ipc");

let unlisten: UnlistenFn;
const receiver = new ReadableStream({
  type: "bytes",
  async start(controller) {
    unlisten = await listen("bin-ipc:ready", async () => {
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

const sender = new WritableStream<Uint8Array>({
  async write(chunk) {
    await fetch(`${localhost}/push`, {
      method: "POST",
      body: chunk,
    });
  },
});

export const BinIPCChannel = createContext({
  receiver,
  sender,
});

render(
  <BinIPCChannel.Provider value={{ sender, receiver }}>
    <App />
  </BinIPCChannel.Provider>,
  document.body,
);
