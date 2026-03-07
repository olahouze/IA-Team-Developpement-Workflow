import { invoke } from '@tauri-apps/api/core';

/**
 * Check URL reachability via Tauri backend command (avoids CORS issues).
 */
export async function isUrlReachable(url: string): Promise<boolean> {
    try {
        return await invoke<boolean>('check_url', { url });
    } catch {
        return false;
    }
}
