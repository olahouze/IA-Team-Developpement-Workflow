import { describe, it, expect } from 'vitest';
import { isWorkerReady } from '../workerTest';

describe('isWorkerReady', () => {
  it('returns true for "worker started"', () => {
    expect(isWorkerReady('[Worker] worker started')).toBe(true);
  });

  it('returns true for "Worker node started"', () => {
    expect(isWorkerReady('[Worker] Worker node started on group default')).toBe(true);
  });

  it('returns true for "[Worker]" with "started"', () => {
    expect(isWorkerReady('[Worker] main loop started')).toBe(true);
  });

  it('returns true for "listening for jobs"', () => {
    expect(isWorkerReady('[Worker] listening for jobs on queue default')).toBe(true);
  });

  it('returns true for "connected to job queue"', () => {
    expect(isWorkerReady('[Worker] connected to job queue')).toBe(true);
  });

  it('returns false for server logs', () => {
    expect(isWorkerReady('[Server] listening on 0.0.0.0:8000')).toBe(false);
  });

  it('returns false for empty string', () => {
    expect(isWorkerReady('')).toBe(false);
  });

  it('returns false for unrelated worker log', () => {
    expect(isWorkerReady('[Worker] processing job abc-123')).toBe(false);
  });
});
