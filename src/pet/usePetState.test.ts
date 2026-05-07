import { describe, it, expect } from 'vitest';
import { computeAggregate } from './usePetState';

describe('computeAggregate', () => {
  it('returns sleeping when no sessions', () => {
    expect(computeAggregate([])).toBe('sleeping');
  });

  it('picks thinking as highest priority', () => {
    const sessions = [
      { status: 'idle' },
      { status: 'thinking' },
      { status: 'waiting' },
    ];
    expect(computeAggregate(sessions)).toBe('thinking');
  });

  it('picks processing when thinking not present', () => {
    const sessions = [
      { status: 'idle' },
      { status: 'processing' },
      { status: 'waiting' },
    ];
    expect(computeAggregate(sessions)).toBe('processing');
  });

  it('maps compacting to processing', () => {
    const sessions = [{ status: 'compacting' }];
    expect(computeAggregate(sessions)).toBe('processing');
  });

  it('returns idle for idle-only sessions', () => {
    const sessions = [{ status: 'idle' }, { status: 'idle' }];
    expect(computeAggregate(sessions)).toBe('idle');
  });

  it('handles unknown status gracefully', () => {
    const sessions = [{ status: 'unknown' }, { status: 'idle' }];
    expect(computeAggregate(sessions)).toBe('idle');
  });
});
