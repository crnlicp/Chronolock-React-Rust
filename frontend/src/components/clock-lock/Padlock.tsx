import { useId } from 'react';

/**
 * Vector version of the original padlock artwork (lock.png, 600x600). All
 * gradients are built from colour profiles sampled from that image:
 * chrome shackle, brushed-steel body with a top tab and blue status light,
 * recessed bezel, blue combination dial (0-35) and a chrome knob.
 *
 * `open` keeps the same steel case and shackle (lifted, left leg free) but
 * replaces the combination face with an "unlocked" face.
 */

const CX = 311;
const BODY_CY = 410;
const BODY_R = 185;
const DIAL_CY = 410;
const ARC_CY = 188; // centre of the shackle arc
const ARC_R = 114; // centre line of the shackle (outer 139, inner 89)
const ARC_W = 50;
const TICKS = Array.from({ length: 40 }, (_, i) => i);
const NUMBERS = [0, 5, 10, 15, 20, 25, 30, 35];

const INNER_R = ARC_R - ARC_W / 2; // 89
const OUTER_R = ARC_R + ARC_W / 2; // 139

/** [radius, colour] pairs -> stops for a radial gradient of radius `r` (arch). */
function stops(pairs: [number, string][], r: number) {
  return pairs.map(([d, c]) => <stop key={d} offset={Math.min(1, d / r)} stopColor={c} />);
}

/** The same profile across a straight leg: offset 0 = inner edge, 1 = outer edge. */
function legStops(pairs: [number, string][]) {
  return pairs.map(([d, c]) => (
    <stop key={d} offset={Math.min(1, Math.max(0, (d - INNER_R) / ARC_W))} stopColor={c} />
  ));
}

// Sampled across a shackle leg (inner edge -> outer edge).
const LEG: [number, string][] = [
  [89, '#2f312f'], [91, '#444744'], [93, '#535654'], [97, '#616462'], [101, '#939594'],
  [103, '#b3b5b4'], [107, '#cdcece'], [111, '#d0d1d1'], [115, '#e0e0e0'], [119, '#e8e8e8'],
  [123, '#cdcece'], [127, '#a2a4a3'], [129, '#535654'], [131, '#5a5d5b'], [135, '#4f5250'], [139, '#555655'],
];
// Sampled down the top of the arch (lit from above): highlight, groove, bright outer band.
const ARCH: [number, string][] = [
  [89, '#5a5d5b'], [94, '#636765'], [100, '#6d6f6d'], [104, '#808280'], [106, '#b0b2b1'],
  [108, '#c5c6c6'], [110, '#8a8c8b'], [112, '#474a47'], [114, '#444744'], [116, '#c8c9c8'],
  [118, '#fcfdfc'], [122, '#f9f9f9'], [126, '#f0f1f1'], [130, '#e8e8e8'], [134, '#cdcece'],
  [138, '#a2a4a3'], [140, '#878a88'],
];

export const Padlock = ({ open = false }: { open?: boolean }) => {
  const uid = useId().replace(/[^a-zA-Z0-9_-]/g, '');
  const id = (n: string) => `${n}-${uid}`;
  const url = (n: string) => `url(#${id(n)})`;
  const L = CX - ARC_R;
  const R = CX + ARC_R;
  const arch = `M${L} ${ARC_CY} A${ARC_R} ${ARC_R} 0 0 1 ${R} ${ARC_CY}`;
  // Leg ends before the lift. Open: left leg free (ends at y 229 after the 15px lift).
  const leftEnd = open ? 244 : 300;
  const rightEnd = open ? 330 : 300;

  return (
    <svg viewBox="0 0 600 600" aria-hidden="true">
      <defs>
        <radialGradient id={id('leg')} gradientUnits="userSpaceOnUse" cx={CX} cy={ARC_CY} r="140">
          {stops(LEG, 140)}
        </radialGradient>
        <radialGradient id={id('arch')} gradientUnits="userSpaceOnUse" cx={CX} cy={ARC_CY} r="140">
          {stops(ARCH, 140)}
        </radialGradient>
        <linearGradient id={id('legL')} gradientUnits="userSpaceOnUse" x1={CX - INNER_R} y1="0" x2={CX - OUTER_R} y2="0">
          {legStops(LEG)}
        </linearGradient>
        <linearGradient id={id('legR')} gradientUnits="userSpaceOnUse" x1={CX + INNER_R} y1="0" x2={CX + OUTER_R} y2="0">
          {legStops(LEG)}
        </linearGradient>
        <filter id={id('ground')} x="-30%" y="-200%" width="160%" height="500%">
          <feGaussianBlur stdDeviation="7" />
        </filter>
        <linearGradient id={id('archFade')} gradientUnits="userSpaceOnUse" x1="0" y1="100" x2="0" y2="182">
          <stop offset="0" stopColor="#fff" />
          <stop offset="1" stopColor="#000" />
        </linearGradient>
        <mask id={id('archMask')} maskUnits="userSpaceOnUse" x="0" y="0" width="600" height="600">
          <rect width="600" height="600" fill={url('archFade')} />
        </mask>
        {/* the free end of the open shackle: inner notch with a small lip */}
        <mask id={id('notch')} maskUnits="userSpaceOnUse" x="0" y="0" width="600" height="600">
          <rect width="600" height="600" fill="#fff" />
          {open && <path d={`M${L + 14} 206 H${L + 26} V219 H${L + 14} Z`} fill="#000" />}
        </mask>

        {/* body: sphere lit from above (fitted to the sampled grid) */}
        <radialGradient id={id('body')} gradientUnits="userSpaceOnUse" cx={CX} cy="230" r="380">
          <stop offset="0" stopColor="#f4f4f4" />
          <stop offset="0.15" stopColor="#eeeeee" />
          <stop offset="0.3" stopColor="#d6d7d6" />
          <stop offset="0.43" stopColor="#acadac" />
          <stop offset="0.5" stopColor="#8f9190" />
          <stop offset="0.6" stopColor="#6a6d6b" />
          <stop offset="0.72" stopColor="#4f524f" />
          <stop offset="0.85" stopColor="#353836" />
          <stop offset="1" stopColor="#1f211f" />
        </radialGradient>
        <radialGradient id={id('bodyEdge')} gradientUnits="userSpaceOnUse" cx={CX} cy={BODY_CY} r={BODY_R}>
          <stop offset="0.85" stopColor="#000" stopOpacity="0" />
          <stop offset="1" stopColor="#000" stopOpacity="0.2" />
        </radialGradient>
        <linearGradient id={id('tab')} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#fbfbfb" />
          <stop offset="1" stopColor="#dedfdf" />
        </linearGradient>

        {/* dial */}
        <linearGradient id={id('bezel')} gradientUnits="userSpaceOnUse" x1="0" y1="272" x2="0" y2="548">
          <stop offset="0" stopColor="#c4c8cb" />
          <stop offset="0.45" stopColor="#6d6f6d" />
          <stop offset="1" stopColor="#353836" />
        </linearGradient>
        <radialGradient id={id('face')} gradientUnits="userSpaceOnUse" cx={CX} cy="400" r="125">
          <stop offset="0.45" stopColor="#55acf8" />
          <stop offset="0.6" stopColor="#4a9cf3" />
          <stop offset="0.7" stopColor="#3b8bee" />
          <stop offset="0.78" stopColor="#3480e8" />
          <stop offset="0.85" stopColor="#2b66db" />
          <stop offset="0.9" stopColor="#2450cc" />
          <stop offset="0.96" stopColor="#1f42ba" />
          <stop offset="1" stopColor="#183a9e" />
        </radialGradient>
        <radialGradient id={id('openFace')} gradientUnits="userSpaceOnUse" cx={CX} cy="395" r="125">
          <stop offset="0" stopColor="#6ef0c0" />
          <stop offset="0.45" stopColor="#34d399" />
          <stop offset="0.75" stopColor="#10b981" />
          <stop offset="0.92" stopColor="#047857" />
          <stop offset="1" stopColor="#064e3b" />
        </radialGradient>
        <linearGradient id={id('gloss')} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#fff" stopOpacity="0.28" />
          <stop offset="1" stopColor="#fff" stopOpacity="0" />
        </linearGradient>
        <linearGradient id={id('chromeRing')} gradientUnits="userSpaceOnUse" x1="0" y1="354" x2="0" y2="466">
          <stop offset="0" stopColor="#ffffff" />
          <stop offset="0.3" stopColor="#e8eaea" />
          <stop offset="0.5" stopColor="#9ca9ae" />
          <stop offset="0.75" stopColor="#5d6f80" />
          <stop offset="1" stopColor="#26416a" />
        </linearGradient>
        <linearGradient id={id('knob')} gradientUnits="userSpaceOnUse" x1="0" y1="369" x2="0" y2="451">
          <stop offset="0" stopColor="#3f4442" />
          <stop offset="0.15" stopColor="#4c4f4d" />
          <stop offset="0.5" stopColor="#737573" />
          <stop offset="0.85" stopColor="#9b9d9b" />
          <stop offset="1" stopColor="#5a6466" />
        </linearGradient>
        <filter id={id('soft')} x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="4" />
        </filter>
        <filter id={id('glow')} x="-200%" y="-100%" width="500%" height="300%">
          <feGaussianBlur stdDeviation="2.5" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
        <path id={id('labelArc')} d={`M${CX - 88} ${DIAL_CY} A88 88 0 0 0 ${CX + 88} ${DIAL_CY}`} />
      </defs>

      {/* ground shadow */}
      <ellipse cx={CX} cy="588" rx="96" ry="7" fill="#000" opacity="0.4" filter={url('ground')} />

      {/* shackle: straight legs + arch */}
      <g mask={url('notch')}>
        <g transform={open ? 'translate(0 -15)' : undefined}>
          <rect x={CX - OUTER_R} y={ARC_CY - 1} width={ARC_W} height={leftEnd - ARC_CY + 1} rx={open ? 3 : 0} fill={url('legL')} />
          <rect x={CX + INNER_R} y={ARC_CY - 1} width={ARC_W} height={rightEnd - ARC_CY + 1} fill={url('legR')} />
          <path d={arch} fill="none" stroke={url('leg')} strokeWidth={ARC_W} />
          <path d={arch} fill="none" stroke={url('arch')} strokeWidth={ARC_W} mask={url('archMask')} />
        </g>
      </g>

      {/* body */}
      <circle cx={CX} cy={BODY_CY} r={BODY_R} fill={url('body')} />
      <circle cx={CX} cy={BODY_CY} r={BODY_R} fill={url('bodyEdge')} />

      {/* top tab with status light */}
      <path
      d={`M${CX - 70} 250
          Q${CX} 222 ${CX + 70} 250
          Q${CX + 65} 255 ${CX + 55} 270
          M${CX - 55} 270
          Q${CX - 65} 255 ${CX - 70} 250`}
      fill={url('tab')}
      stroke="#b9bbba"
      strokeWidth="1.2"
/>
      <rect
        x={CX - 3}
        y="240"
        width="7"
        height="19"
        rx="3"
        fill={open ? '#34d399' : '#63ccfe'}
        filter={url('glow')}
      />

      {/* recessed bezel and dial edge */}
      <circle cx={CX} cy={DIAL_CY} r="147" fill={url('bezel')} />
      <circle cx={CX} cy={DIAL_CY} r="147" fill="none" stroke="#626962" strokeOpacity="0.7" strokeWidth="1.5" />
      <circle cx={CX} cy={DIAL_CY} r="124" fill={open ? '#053d2e' : '#0f2458'} />

      {open ? (
        <>
          <circle cx={CX} cy={DIAL_CY} r="120" fill={url('openFace')} />
          <path d={`M${CX - 112} ${DIAL_CY - 4} A112 112 0 0 1 ${CX + 112} ${DIAL_CY - 4} Z`} fill={url('gloss')} />
          <g stroke="#ecfdf5" strokeLinecap="round">
            {TICKS.map((i) => (
              <line
                key={i}
                x1={CX}
                y1={DIAL_CY - (i % 5 === 0 ? 100 : 106)}
                x2={CX}
                y2={DIAL_CY - 113}
                strokeWidth={i % 5 === 0 ? 3 : 1.5}
                opacity={i % 5 === 0 ? 0.85 : 0.55}
                transform={`rotate(${i * 9} ${CX} ${DIAL_CY})`}
              />
            ))}
          </g>
          <circle cx={CX} cy={DIAL_CY - 10} r="54" fill="#064e3b" opacity="0.35" filter={url('soft')} />
          <path
            d={`M${CX - 42} ${DIAL_CY - 12} L${CX - 12} ${DIAL_CY + 18} L${CX + 46} ${DIAL_CY - 42}`}
            fill="none"
            stroke="#ffffff"
            strokeWidth="20"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
          <text fill="#ecfdf5" fontFamily="Arial, Helvetica, sans-serif" fontSize="17" fontWeight="700" letterSpacing="5">
            <textPath href={`#${id('labelArc')}`} startOffset="50%" textAnchor="middle">
              UNLOCKED
            </textPath>
          </text>
        </>
      ) : (
        <>
          <circle cx={CX} cy={DIAL_CY} r="120" fill={url('face')} />
          <path d={`M${CX - 112} ${DIAL_CY - 4} A112 112 0 0 1 ${CX + 112} ${DIAL_CY - 4} Z`} fill={url('gloss')} />
          <g stroke="#a8cff7" strokeLinecap="round">
            {TICKS.map((i) => (
              <line
                key={i}
                x1={CX}
                y1={DIAL_CY - (i % 5 === 0 ? 100 : 106)}
                x2={CX}
                y2={DIAL_CY - 113}
                strokeWidth={i % 5 === 0 ? 3 : 1.5}
                opacity={i % 5 === 0 ? 1 : 0.8}
                transform={`rotate(${i * 9} ${CX} ${DIAL_CY})`}
              />
            ))}
          </g>
          <g fill="#e6f1fd" fontFamily="Arial, Helvetica, sans-serif" fontSize="18" textAnchor="middle">
            {NUMBERS.map((n) => (
              <text key={n} x={CX} y={DIAL_CY - 80} transform={`rotate(${n * 9} ${CX} ${DIAL_CY})`}>
                {n}
              </text>
            ))}
          </g>
          <circle cx={CX} cy={DIAL_CY} r="58" fill="#2a56a8" opacity="0.7" filter={url('soft')} />
          <circle cx={CX} cy={DIAL_CY} r="55" fill={url('chromeRing')} />
          <circle cx={CX} cy={DIAL_CY} r="55" fill="none" stroke="#1d3c6e" strokeOpacity="0.5" strokeWidth="1.5" />
          <circle cx={CX} cy={DIAL_CY} r="41" fill={url('knob')} />
          <circle cx={CX} cy={DIAL_CY} r="41" fill="none" stroke="#2b383c" strokeWidth="2" />
        </>
      )}
    </svg>
  );
};
