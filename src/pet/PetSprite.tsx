import { useState, useEffect, useCallback } from 'react';
import { CharacterConfig, PetAnimationState } from './config';

interface PetSpriteProps {
  state: PetAnimationState;
  config: CharacterConfig;
}

export function PetSprite({ state, config }: PetSpriteProps) {
  const [frameIndex, setFrameIndex] = useState(0);
  const [imgError, setImgError] = useState(false);

  const stateConfig = config.states[state] ?? config.states[config.defaultState];
  const totalFrames = stateConfig.count;

  // Cycle frame index on a timer
  useEffect(() => {
    if (totalFrames <= 1) {
      setFrameIndex(0);
      return;
    }
    setFrameIndex(0);
    const interval = setInterval(() => {
      setFrameIndex(prev => (prev + 1) % totalFrames);
    }, config.frameRate);
    return () => clearInterval(interval);
  }, [state, config.frameRate, totalFrames]);

  const handleError = useCallback(() => setImgError(true), []);

  const absoluteFrame = stateConfig.startFrame + frameIndex;
  const col = absoluteFrame % config.columns;
  const row = Math.floor(absoluteFrame / config.columns);
  const bgX = -(col * config.frameWidth);
  const bgY = -(row * config.frameHeight);

  // Total sprite sheet size
  const totalFramesAll = Object.values(config.states).reduce(
    (max, s) => Math.max(max, s.startFrame + s.count), 0
  );
  const sheetCols = config.columns;
  const sheetRows = Math.ceil(totalFramesAll / sheetCols);

  // Fallback: colored shape when sprite image missing
  if (imgError) {
    const colors: Record<string, string> = {
      thinking: '#fbbf24',
      processing: '#60a5fa',
      waiting: '#a3e635',
      idle: '#94a3b8',
      sleeping: '#6b7280',
    };
    return (
      <div
        className="flex items-center justify-center rounded-full"
        style={{
          width: config.frameWidth,
          height: config.frameHeight,
          backgroundColor: colors[state] ?? colors.sleeping,
          opacity: 0.8,
        }}
      >
        <span className="text-xs font-mono text-white">{state}</span>
      </div>
    );
  }

  return (
    <div
      style={{
        width: config.frameWidth,
        height: config.frameHeight,
        backgroundImage: `url(${config.sprite})`,
        backgroundSize: `${sheetCols * config.frameWidth}px ${sheetRows * config.frameHeight}px`,
        backgroundPositionX: bgX,
        backgroundPositionY: bgY,
        backgroundRepeat: 'no-repeat',
        imageRendering: 'pixelated',
        cursor: 'grab',
      }}
      onError={handleError}
    />
  );
}
