export interface CharacterState {
  startFrame: number;
  count: number;
}

export interface CharacterConfig {
  name: string;
  sprite: string;
  columns: number; // frames per row in the sprite sheet grid
  frameWidth: number;
  frameHeight: number;
  frameRate: number; // ms per frame
  states: Record<string, CharacterState>;
  defaultState: string;
}

export type PetAnimationState = 'thinking' | 'processing' | 'waiting' | 'idle' | 'sleeping';

const ROBOT_CONFIG: CharacterConfig = {
  name: 'robot',
  sprite: '/pets/robot.png',
  columns: 4,
  frameWidth: 128,
  frameHeight: 128,
  frameRate: 500,
  states: {
    idle:       { startFrame: 0, count: 4 },
    thinking:   { startFrame: 4, count: 4 },
    processing: { startFrame: 8, count: 4 },
    waiting:    { startFrame: 12, count: 4 },
    sleeping:   { startFrame: 16, count: 2 },
  },
  defaultState: 'sleeping',
};

/**
 * Load character config by name.
 * v1: returns hardcoded robot config.
 * Future: read from src-tauri/pets/<name>.json at runtime.
 */
export function loadCharacterConfig(_name: string): CharacterConfig {
  return ROBOT_CONFIG;
}
