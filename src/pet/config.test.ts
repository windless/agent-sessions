import { describe, it, expect } from 'vitest';
import { loadCharacterConfig } from './config';

describe('loadCharacterConfig', () => {
  it('returns robot config by default', () => {
    const config = loadCharacterConfig('robot');
    expect(config.name).toBe('robot');
    expect(config.frameWidth).toBe(128);
    expect(config.frameHeight).toBe(128);
    expect(config.frameRate).toBe(500);
  });

  it('returns config with all expected states', () => {
    const config = loadCharacterConfig('anything');
    expect(config.columns).toBe(6);
    expect(config.states.idle.count).toBe(4);
    expect(config.states.thinking.count).toBe(4);
    expect(config.states.processing.count).toBe(4);
    expect(config.states.waiting.count).toBe(4);
    expect(config.states.sleeping.count).toBe(2);
  });

  it('has valid defaultState', () => {
    const config = loadCharacterConfig('robot');
    expect(config.states[config.defaultState]).toBeDefined();
  });

  it('state startFrame values do not overlap', () => {
    const config = loadCharacterConfig('robot');
    const ranges = Object.values(config.states).map(
      s => [s.startFrame, s.startFrame + s.count] as const
    );
    for (let i = 0; i < ranges.length; i++) {
      for (let j = i + 1; j < ranges.length; j++) {
        const [aStart, aEnd] = ranges[i];
        const [bStart] = ranges[j];
        expect(aEnd).toBeLessThanOrEqual(bStart);
      }
    }
  });
});
