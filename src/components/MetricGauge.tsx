import { Card, CardContent } from "./ui/card";

interface MetricGaugeProps {
  value: number;
  label: string;
  sublabel: string;
  color: string;
  size?: number;
}

export function MetricGauge({ value, label, sublabel, color, size = 180 }: MetricGaugeProps) {
  const clampedValue = Math.min(100, Math.max(0, value));
  const stroke = 14;
  const center = size / 2;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const arcLength = (240 / 360) * circumference;
  const fillOffset = arcLength - (clampedValue / 100) * arcLength;
  const pulse = clampedValue > 80;

  return (
    <Card
      className="min-w-[160px] border-border/70"
      style={{
        boxShadow: `0 0 24px ${color}26`,
      }}
    >
      <CardContent className="flex flex-col items-center justify-center p-6">
        <style>
          {`@keyframes pulse-ring { 0% { opacity: 1; } 50% { opacity: 0.7; } 100% { opacity: 1; } }`}
        </style>
        <div className="relative" style={{ width: size, height: size }}>
          <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
            <circle
              cx={center}
              cy={center}
              r={radius}
              fill="none"
              stroke="var(--color-gauge-bg, #374151)"
              strokeWidth={stroke}
              strokeDasharray={`${arcLength} ${circumference}`}
              strokeDashoffset={0}
              strokeLinecap="round"
              transform={`rotate(150 ${center} ${center})`}
            />
            <circle
              cx={center}
              cy={center}
              r={radius}
              fill="none"
              stroke={color}
              strokeWidth={stroke}
              strokeDasharray={`${arcLength} ${circumference}`}
              strokeDashoffset={fillOffset}
              strokeLinecap="round"
              transform={`rotate(150 ${center} ${center})`}
              style={{
                transition: "stroke-dashoffset 1s cubic-bezier(0.4,0,0.2,1)",
                animation: pulse ? "pulse-ring 1.6s ease-in-out infinite" : undefined,
              }}
            />
          </svg>
          <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
            <div className="text-3xl font-bold leading-none">{Math.round(clampedValue)}%</div>
            <div className="mt-2 text-xs text-muted-foreground">{sublabel}</div>
          </div>
        </div>
        <div className="mt-2 text-center text-sm font-semibold tracking-wide">{label}</div>
      </CardContent>
    </Card>
  );
}
