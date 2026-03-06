export function isServerReady(logLine: string): boolean {
    return logLine.toLowerCase().includes('server running on') ||
        logLine.toLowerCase().includes('listening on') ||
        logLine.toLowerCase().includes('windmill server started') ||
        (logLine.toLowerCase().includes('health check') && !logLine.toLowerCase().includes('no workers alive')) ||
        logLine.includes('Running on');
}
