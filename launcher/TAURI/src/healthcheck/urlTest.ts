export async function isUrlReachable(url: string): Promise<boolean> {
    try {
        const response = await fetch(url, { method: 'GET' });
        return response.ok || response.status < 500;
    } catch (error) {
        return false;
    }
}
