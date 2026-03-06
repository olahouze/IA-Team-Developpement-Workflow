export function isPgReady(logLine: string): boolean {
    return logLine.includes('database system is ready to accept connections') || logLine.includes('starting PostgreSQL');
}
