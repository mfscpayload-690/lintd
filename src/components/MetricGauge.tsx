interface MetricGaugeProps {
  value: number; // 0-100 percentage
  label: string; // e.g., "CPU", "RAM"
  sublabel: string; // e.g., "65%" or "10.2 GB / 15.6 GB"
  color: string; // hex color
  size?: number; // default 160
}

export function MetricGauge({ value, label, sublabel, color, size = 160 }: MetricGaugeProps) {
  // Clamp value between 0 and 100
  const clampedValue = Math.min(100, Math.max(0, value));

  // SVG dimensions
  const radius = (size - 16) / 2; // 16px padding
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (clampedValue / 100) * circumference;

  return (
    <div className="flex flex-col items-center gap-2">
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="drop-shadow-sm">
        {/* Background ring */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--color-gauge-bg, #e5e7eb)"
          strokeWidth="4"
        />

        {/* Foreground arc (usage) */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={color}
          strokeWidth="4"
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
          style={{
            transform: `rotate(-90deg)`,
            transformOrigin: `${size / 2}px ${size / 2}px`,
            transition: "stroke-dashoffset 0.5s ease-in-out",
          }}
        />

        {/* Center text: percentage */}
        <text
          x={size / 2}
          y={size / 2 - 8}
          textAnchor="middle"
          className="fill-foreground"
          style={{
            fontSize: `${size * 0.3}px`,
            fontWeight: "700",
          }}
        >
          {clampedValue.toFixed(0)}%
        </text>

        {/* Center text: sublabel (smaller) */}
        <text
          x={size / 2}
          y={size / 2 + 16}
          textAnchor="middle"
          className="fill-muted-foreground"
          style={{
            fontSize: `${size * 0.12}px`,
            fontWeight: "500",
          }}
        >
          {sublabel}
        </text>
      </svg>

      {/* Label below */}
      <div className="text-sm font-medium text-foreground">{label}</div>
    </div>
  );
}
