import { describe, it, expect } from 'vitest';
import { isServerReady } from '../serverTest';

describe('isServerReady', () => {
  it('returns true for "listening on 0.0.0.0:8000"', () => {
    expect(isServerReady('[Server] listening on 0.0.0.0:8000')).toBe(true);
  });

  it('returns true for "Listening on" (case-insensitive)', () => {
    expect(isServerReady('[Server] Listening on 0.0.0.0:8000')).toBe(true);
  });

  it('returns true for "server running on"', () => {
    expect(isServerReady('[Server] server running on http://0.0.0.0:8000')).toBe(true);
  });

  it('returns true for "windmill server started"', () => {
    expect(isServerReady('[Server] Windmill Server Started')).toBe(true);
  });

  it('returns true for "started server on"', () => {
    expect(isServerReady('[Server] started server on 0.0.0.0:8000')).toBe(true);
  });

  it('returns true for "Running on"', () => {
    expect(isServerReady('[Server] Running on http://0.0.0.0:8000')).toBe(true);
  });

  it('returns false for unrelated log lines', () => {
    expect(isServerReady('[Worker] worker started')).toBe(false);
  });

  it('returns false for empty string', () => {
    expect(isServerReady('')).toBe(false);
  });

  it('returns false for config spam', () => {
    expect(isServerReady('Loaded WORKER_GROUP setting to None')).toBe(false);
  });
});
