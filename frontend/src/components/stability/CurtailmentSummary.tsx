import SectionModule from "@/components/ui/SectionModule";

interface CurtailmentSummaryProps {
  ytdGwh: number;
  forecastGwh: number;
  windGwh: number;
  solarGwh: number;
  isEstimate?: boolean;
}

interface StatBoxProps {
  label: string;
  value: number;
  unit: string;
  icon: string;
}

function StatBox({ label, value, unit, icon }: StatBoxProps) {
  return (
    <div className="bg-surface-container-lowest p-5 rounded-xl">
      <div className="flex items-center gap-2 mb-3">
        <span className="material-symbols-outlined text-[18px] text-on-surface-variant">
          {icon}
        </span>
        <p className="text-[0.6875rem] uppercase tracking-widest text-on-surface-variant">
          {label}
        </p>
      </div>
      <p className="font-headline text-2xl font-extrabold text-on-surface">
        {value.toFixed(0)}
        <span className="text-sm text-on-surface-variant ml-1">{unit}</span>
      </p>
    </div>
  );
}

export default function CurtailmentSummary({
  ytdGwh,
  forecastGwh,
  windGwh,
  solarGwh,
  isEstimate,
}: CurtailmentSummaryProps) {
  return (
    <SectionModule
      title="Curtailment Analysis"
      subtitle="Year-to-date renewable energy curtailment statistics"
    >
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
        <StatBox
          label="YTD Curtailment"
          value={ytdGwh}
          unit="GWh"
          icon="trending_down"
        />
        <StatBox
          label="Forecast Annual"
          value={forecastGwh}
          unit="GWh"
          icon="event_upcoming"
        />
        <StatBox
          label="Wind Reduction"
          value={windGwh}
          unit="GWh"
          icon="air"
        />
        <StatBox
          label="Solar Reduction"
          value={solarGwh}
          unit="GWh"
          icon="wb_sunny"
        />
      </div>
      {isEstimate && (
        <p className="text-[10px] text-on-surface-variant italic mt-3 text-center">
          * Estimates derived from residual demand model. PSE A77 subscription
          required for official curtailment data.
        </p>
      )}
    </SectionModule>
  );
}
