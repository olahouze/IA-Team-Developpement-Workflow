export function isWorkerReady(logLine: string): boolean {
    // Le log du worker ou du server indiquant un worker en bonne santé
    return logLine.includes('worker started') ||
        logLine.includes('Worker node started') ||
        (logLine.includes('[Worker]') && logLine.includes('started')) ||
        (logLine.includes('health check') && logLine.includes('healthy status'));
}
