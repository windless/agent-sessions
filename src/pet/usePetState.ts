import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PetAnimationState } from './config';

interface SessionsResponse {
  sessions: Array<{ status: string }>;
  totalCount: number;
}

const POLL_INTERVAL = 2000;

const STATUS_PRIORITY: Record<string, number> = {
  thinking: 0,
  processing: 1,
  compacting: 2,
  waiting: 3,
  idle: 4,
};

export function computeAggregate(sessions: Array<{ status: string }>): PetAnimationState {
  if (sessions.length === 0) return 'sleeping';

  let best: PetAnimationState = 'idle';
  let bestPriority = Infinity;

  for (const s of sessions) {
    const priority = STATUS_PRIORITY[s.status] ?? 5;
    if (priority < bestPriority) {
      bestPriority = priority;
      // Map the SessionStatus to PetAnimationState
      switch (s.status) {
        case 'thinking': best = 'thinking'; break;
        case 'processing': best = 'processing'; break;
        case 'compacting': best = 'processing'; break;
        case 'waiting': best = 'waiting'; break;
        case 'idle': best = 'idle'; break;
        default: best = 'idle';
      }
    }
  }
  return best;
}

export function usePetState() {
  const [state, setState] = useState<PetAnimationState>('sleeping');
  const [activeCount, setActiveCount] = useState(0);
  const stateRef = useRef<PetAnimationState>('sleeping');

  useEffect(() => {
    const poll = async () => {
      try {
        const response = await invoke<SessionsResponse>('get_all_sessions');
        const aggregate = computeAggregate(response.sessions);
        setActiveCount(response.totalCount);
        if (aggregate !== stateRef.current) {
          stateRef.current = aggregate;
          setState(aggregate);
        }
      } catch {
        // Silently retain last known state on error
      }
    };

    poll(); // Immediate first poll
    const interval = setInterval(poll, POLL_INTERVAL);
    return () => clearInterval(interval);
  }, []);

  return { state, activeCount };
}
