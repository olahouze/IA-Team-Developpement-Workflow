export function isWorkerReady(logLine: string): boolean {
    const lower = logLine.toLowerCase();
    return lower.includes('worker started') ||
        lower.includes('worker node started') ||
        (logLine.includes('[Worker]') && lower.includes('started')) ||
        lower.includes('listening for jobs') ||
        lower.includes('connected to job queue');
}
