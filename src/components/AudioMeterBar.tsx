import { cn } from "@/lib/utils";
import type { AudioMeter } from "@/lib/tauri";

type Props = {
  meter: AudioMeter;
  className?: string;
};

// Convertit une valeur dB typique [-60, 0] en pourcentage [0, 100].
function dbToPercent(db: number): number {
  if (!isFinite(db)) return 0;
  const clamped = Math.max(-60, Math.min(0, db));
  return ((clamped + 60) / 60) * 100;
}

export function AudioMeterBar({ meter, className }: Props) {
  const rms = dbToPercent(meter.rms_db);
  const peak = dbToPercent(meter.peak_db);
  const peakColor =
    meter.peak_db > -3
      ? "bg-red-500"
      : meter.peak_db > -12
        ? "bg-amber-500"
        : "bg-green-500";

  return (
    <div className={cn("w-full", className)}>
      <div className="relative h-3 w-full overflow-hidden rounded-full bg-muted">
        <div
          className={cn("absolute left-0 top-0 h-full transition-all", peakColor)}
          style={{ width: `${rms}%`, opacity: 0.9 }}
        />
        <div
          className="absolute top-0 h-full w-0.5 bg-foreground/70"
          style={{ left: `${peak}%` }}
        />
      </div>
      <div className="mt-1 flex justify-between text-[10px] text-muted-foreground tabular-nums">
        <span>RMS {meter.rms_db.toFixed(0)} dB</span>
        <span>peak {meter.peak_db.toFixed(0)} dB</span>
      </div>
    </div>
  );
}
