export function isServerReady(logLine: string): boolean {
    const lower = logLine.toLowerCase();
    return lower.includes('listening on 0.0.0.0:8000') ||
        lower.includes('server running on') ||
        lower.includes('windmill server started') ||
        lower.includes('listening on') ||
        lower.includes('started server on') ||
        logLine.includes('Running on');
}
