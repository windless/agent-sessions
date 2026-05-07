import { useCallback } from 'react';
import { usePetState } from './usePetState';
import { PetSprite } from './PetSprite';
import { loadCharacterConfig } from './config';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function PetApp() {
  const { state } = usePetState();
  const config = loadCharacterConfig('robot');

  const handleMouseDown = useCallback(() => {
    getCurrentWindow().startDragging();
  }, []);

  return (
    <div
      data-tauri-drag-region
      className="fixed inset-0 bg-transparent"
      style={{ cursor: 'grab' }}
      onMouseDown={handleMouseDown}
    >
      <PetSprite state={state} config={config} />
    </div>
  );
}
