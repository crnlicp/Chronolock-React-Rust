import { useState, useEffect, useRef } from 'react';

interface TimerManager {
  timers: Map<string, NodeJS.Timeout>;
  callbacks: Map<string, () => void>;
}

// Global timer manager to share across all components
let globalTimerManager: TimerManager | null = null;

const getTimerManager = (): TimerManager => {
  if (!globalTimerManager) {
    globalTimerManager = {
      timers: new Map(),
      callbacks: new Map(),
    };
  }
  return globalTimerManager;
};

/**
 * Custom hook to efficiently track when a chronolock becomes unlocked
 * Uses a global timer manager to prevent multiple timers for the same unlock time
 * and cleans up automatically when components unmount
 */
export const useLockTimer = (lockTime: number | undefined, id: string) => {
  const [isLocked, setIsLocked] = useState(() => {
    if (!lockTime) return true;
    return Date.now() / 1000 < lockTime;
  });
  
  const timerManager = useRef(getTimerManager());
  const callbackRef = useRef<() => void>();

  useEffect(() => {
    if (!lockTime) {
      setIsLocked(true);
      return;
    }

    const now = Date.now() / 1000;
    const initialIsLocked = now < lockTime;
    setIsLocked(initialIsLocked);

    // If already unlocked, no need to set a timer
    if (!initialIsLocked) {
      return;
    }

    // Create unique key for this unlock time
    const timerKey = `unlock_${lockTime}`;
    
    // Define callback for this specific component
    const callback = () => {
      setIsLocked(false);
    };
    callbackRef.current = callback;

    // Check if we already have a timer for this unlock time
    if (!timerManager.current.timers.has(timerKey)) {
      // Calculate time until unlock (in milliseconds)
      const timeUntilUnlock = (lockTime - now) * 1000;
      
      // Set timer only if unlock time is in the future
      if (timeUntilUnlock > 0) {
        const timer = setTimeout(() => {
          // Notify all components waiting for this unlock time
          const callbacks = Array.from(timerManager.current.callbacks.entries())
            .filter(([key]) => key.startsWith(timerKey))
            .map(([, cb]) => cb);
          
          callbacks.forEach(cb => cb());
          
          // Clean up timer
          timerManager.current.timers.delete(timerKey);
          // Clean up all callbacks for this timer
          Array.from(timerManager.current.callbacks.keys())
            .filter(key => key.startsWith(timerKey))
            .forEach(key => timerManager.current.callbacks.delete(key));
        }, timeUntilUnlock);

        timerManager.current.timers.set(timerKey, timer);
      }
    }

    // Register this component's callback
    const callbackKey = `${timerKey}_${id}`;
    timerManager.current.callbacks.set(callbackKey, callback);

    // Cleanup function
    return () => {
      // Remove this component's callback
      const callbackKey = `${timerKey}_${id}`;
      timerManager.current.callbacks.delete(callbackKey);

      // If this was the last callback for this timer, clean up the timer
      const remainingCallbacks = Array.from(timerManager.current.callbacks.keys())
        .filter(key => key.startsWith(timerKey));
      
      if (remainingCallbacks.length === 0) {
        const timer = timerManager.current.timers.get(timerKey);
        if (timer) {
          clearTimeout(timer);
          timerManager.current.timers.delete(timerKey);
        }
      }
    };
  }, [lockTime, id]);

  return isLocked;
};