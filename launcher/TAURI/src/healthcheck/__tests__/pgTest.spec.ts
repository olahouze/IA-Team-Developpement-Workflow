import { describe, it, expect } from 'vitest';
import { isPgReady } from '../pgTest';

describe('isPgReady', () => {
  it('returns true for standard PG ready message', () => {
    expect(isPgReady('LOG:  database system is ready to accept connections')).toBe(true);
  });

  it('returns true for "starting PostgreSQL" message', () => {
    expect(isPgReady('[PostgreSQL] starting PostgreSQL on port 5432')).toBe(true);
  });

  it('returns true when message is from Tauri backend', () => {
    expect(isPgReady('[PostgreSQL] database system is ready to accept connections (1500ms)')).toBe(true);
  });

  it('returns false for unrelated log lines', () => {
    expect(isPgReady('[Server] Windmill server started')).toBe(false);
  });

  it('returns false for empty string', () => {
    expect(isPgReady('')).toBe(false);
  });

  it('returns false for partial matches', () => {
    expect(isPgReady('database system is shutting down')).toBe(false);
  });
});
