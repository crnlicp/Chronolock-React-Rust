import { Clock } from './Clock';
import { Padlock } from './Padlock';

interface Props {
  /** Unlock time. */
  target: Date;
  /** Show the open padlock instead of the countdown. */
  unlocked?: boolean;
  /** CSS width, e.g. 320 or '100%'. */
  size?: number | string;
}

/** The Chronolock padlock: a live countdown over its dial, or open once unlocked. */
export const ClockLock = ({ target, unlocked, size = '100%' }: Props) => (
  <div
    className="clock-lock"
    style={{ width: size, maxWidth: '100%' }}
    role="img"
    aria-label={unlocked ? 'Unlocked' : `Locked until ${target.toLocaleString()}`}
  >
    <Padlock open={unlocked} />
    {!unlocked && <Clock targetDate={target} className="clock-window" />}
  </div>
);
