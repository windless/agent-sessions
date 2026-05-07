import { usePetState } from './usePetState';
import { PetSprite } from './PetSprite';
import { loadCharacterConfig } from './config';

export function PetApp() {
  const { state } = usePetState();
  const config = loadCharacterConfig('robot');

  return (
    <div
      data-tauri-drag-region
      className="fixed inset-0 bg-transparent"
      style={{ cursor: 'grab' }}
    >
      <PetSprite state={state} config={config} />
    </div>
  );
}
