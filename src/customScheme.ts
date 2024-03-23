import { platform } from "@tauri-apps/api/os";

export async function customSchemeLocalhost(scheme: string) {
    const name = await platform();
    if (name === "win32") {
        return `https://${scheme}.localhost`;
    }

    return `${scheme}://localhost`;
}