"use client";

interface GaugeChartProps {
  value: number;
  level: string;
  stabilityMargin: number;
  congestionProbability: number;
}

function arcPath(cx: number, cy: number, r: number, startAngle: number, endAngle: number): string {
  const start = polarToCartesian(cx, cy, r, endAngle);
  const end = polarToCartesian(cx, cy, r, startAngle);
  const largeArc = endAngle - startAngle > 180 ? 1 : 0;
  return `M ${start.x} ${start.y} A ${r} ${r} 0 ${largeArc} 0 ${end.x} ${end.y}`;
}

function polarToCartesian(cx: number, cy: number, r: number, angleDeg: number) {
  const rad = ((angleDeg - 90) * Math.PI) / 180.0;
  return {
    x: cx + r * Math.cos(rad),
    y: cy + r * Math.sin(rad),
  };
}

function getArcColor(value: number): string {
  if (value < 50) return "#76d6d5"; // primary
  if (value <= 75) return "#ffb692"; // tertiary
  return "#ffb4ab"; // error
}

function getLevelColor(level: string): string {
  switch (level.toUpperCase()) {
    case "LOW":
      return "text-emerald-400";
    case "MODERATE":
      return "text-primary";
    case "ELEVATED":
      return "text-tertiary";
    case "CRITICAL":
      return "text-error";
    default:
      return "text-on-surface-variant";
  }
}

export default function GaugeChart({
  value,
  level,
  stabilityMargin,
  congestionProbability,
}: GaugeChartProps) {
  const cx = 100;
  const cy = 100;
  const r = 75;
  const startAngle = 135;
  const totalArc = 270;
  const valueAngle = startAngle + (value / 100) * totalArc;

  const bgPath = arcPath(cx, cy, r, startAngle, startAngle + totalArc);
  const valuePath = arcPath(cx, cy, r, startAngle, valueAngle);
  const color = getArcColor(value);

  return (
    <div className="bg-surface-container p-6 rounded-xl">
      <div className="flex justify-center">
        <svg viewBox="0 0 200 170" className="w-full max-w-[280px]">
          {/* Background arc */}
          <path
            d={bgPath}
            fill="none"
            stroke="#222a3d"
            strokeWidth="12"
            strokeLinecap="round"
          />
          {/* Value arc */}
          <path
            d={valuePath}
            fill="none"
            stroke={color}
            strokeWidth="12"
            strokeLinecap="round"
          />
          {/* Center value */}
          <text
            x={cx}
            y={cy - 5}
            textAnchor="middle"
            className="font-headline"
            fill="#dae2fd"
            fontSize="36"
            fontWeight="800"
          >
            {value.toFixed(1)}
          </text>
          {/* Level label */}
          <text
            x={cx}
            y={cy + 20}
            textAnchor="middle"
            fill="#bdc9c8"
            fontSize="11"
            fontWeight="500"
            letterSpacing="0.1em"
          >
            {level.toUpperCase()}
          </text>
          {/* CRI label */}
          <text
            x={cx}
            y={cy + 35}
            textAnchor="middle"
            fill="#879392"
            fontSize="9"
            letterSpacing="0.15em"
          >
            CURTAILMENT RISK INDEX
          </text>
        </svg>
      </div>

      {/* Stat boxes */}
      <div className="grid grid-cols-2 gap-4 mt-4">
        <div className="bg-surface-container-lowest p-4 rounded-xl text-center">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant mb-1">
            Stability Margin
          </p>
          <p className="font-headline text-xl font-extrabold text-on-surface">
            {stabilityMargin.toFixed(1)}
            <span className="text-sm text-on-surface-variant ml-1">GW</span>
          </p>
        </div>
        <div className="bg-surface-container-lowest p-4 rounded-xl text-center">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant mb-1">
            Congestion Prob.
          </p>
          <p className={`font-headline text-xl font-extrabold ${getLevelColor(level)}`}>
            {congestionProbability.toFixed(1)}
            <span className="text-sm opacity-70 ml-0.5">%</span>
          </p>
        </div>
      </div>
    </div>
  );
}
