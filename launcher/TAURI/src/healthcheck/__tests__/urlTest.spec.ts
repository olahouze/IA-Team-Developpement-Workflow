import { describe, it, expect, vi } from 'vitest';

// Mock @tauri-apps/api/core before importing the module under test
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { isUrlReachable } from '../urlTest';
import { invoke } from '@tauri-apps/api/core';

const mockedInvoke = vi.mocked(invoke);

describe('isUrlReachable', () => {
  it('returns true when invoke returns true', async () => {
    mockedInvoke.mockResolvedValueOnce(true);
    const result = await isUrlReachable('http://localhost:8000/');
    expect(result).toBe(true);
    expect(mockedInvoke).toHaveBeenCalledWith('check_url', { url: 'http://localhost:8000/' });
  });

  it('returns false when invoke returns false', async () => {
    mockedInvoke.mockResolvedValueOnce(false);
    const result = await isUrlReachable('http://localhost:8000/');
    expect(result).toBe(false);
  });

  it('returns false when invoke throws', async () => {
    mockedInvoke.mockRejectedValueOnce(new Error('Tauri not available'));
    const result = await isUrlReachable('http://localhost:8000/');
    expect(result).toBe(false);
  });
});
