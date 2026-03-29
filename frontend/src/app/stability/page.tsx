import { getResidual, getCurtailment } from "@/lib/api";
import LiveBadge from "@/components/ui/LiveBadge";
import MetricCard from "@/components/ui/MetricCard";
import SectionModule from "@/components/ui/SectionModule";
import GaugeChart from "@/components/charts/GaugeChart";
import HeatmapGrid from "@/components/charts/HeatmapGrid";
import ScatterPlot from "@/components/charts/ScatterPlot";
import ResidualDemandChart from "@/components/stability/ResidualDemandChart";
import CurtailmentSummary from "@/components/stability/CurtailmentSummary";
import CurtailmentByCause from "@/components/stability/CurtailmentByCause";
import HistoricalChart from "@/components/charts/HistoricalChart";
import DataLoadingCard from "@/components/ui/DataLoadingCard";

export const revalidate = 3600;

export default async function StabilityPage() {
  const [residualResult, curtailmentResult] = await Promise.allSettled([
    getResidual(),
    getCurtailment(),
  ]);
  const residual = residualResult.status === "fulfilled" ? residualResult.value : null;
  const curtailment = curtailmentResult.status === "fulfilled" ? curtailmentResult.value : null;

  if (!residual || residual.data_status === "unavailable") {
    return (
      <DataLoadingCard
        section="Stability"
        message={residual?.message ?? "Fetching from live sources — reload in 30s"}
      />
    );
  }

  return (
    <div className="p-8 space-y-8">
      {/* Page Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="font-headline text-2xl font-bold text-on-surface">
            Stability & OZE Analysis
          </h1>
          <p className="text-sm text-on-surface-variant mt-1">
            Residual demand, curtailment risk, and grid stability metrics
          </p>
        </div>
        <div className="flex items-center gap-3">
          <LiveBadge />
          <button className="bg-linear-to-br from-primary to-primary-container text-on-primary px-4 py-1.5 rounded-lg font-semibold hover:opacity-90 transition-opacity text-sm">
            Export Report
          </button>
        </div>
      </div>

      {/* Section A: CRI Overview */}
      <div className="grid grid-cols-12 gap-6">
        <div className="col-span-12 lg:col-span-5">
          <GaugeChart
            value={residual.cri_value}
            level={residual.cri_level}
            stabilityMargin={residual.stability_margin_gw}
            congestionProbability={residual.congestion_probability_pct}
          />
        </div>
        <div className="col-span-12 lg:col-span-7 grid grid-cols-2 gap-6">
          <MetricCard
            label="Current Residual Demand"
            value={`${residual.current_residual_gw.toFixed(1)}`}
            unit="GW"
            delta={0}
          />
          <MetricCard
            label="Must-Run Floor"
            value={`${residual.must_run_floor_gw.toFixed(1)}`}
            unit="GW"
            delta={0}
          />
          <MetricCard
            label="Stability Margin"
            value={`${residual.stability_margin_gw.toFixed(1)}`}
            unit="GW"
            delta={0}
          />
          <MetricCard
            label="Congestion Probability"
            value={`${residual.congestion_probability_pct.toFixed(1)}`}
            unit="%"
            delta={0}
          />
        </div>
      </div>

      {/* Section B: Residual Demand Profile */}
      <SectionModule
        title="24-Hour Residual Demand Profile"
        subtitle="Hourly residual demand vs must-run generation floor"
      >
        <ResidualDemandChart data={residual.hourly_profile} />
      </SectionModule>

      {/* Section C: Heatmap */}
      <SectionModule
        title="Residual Demand Heatmap"
        subtitle="24-hour × 12-month residual demand intensity"
      >
        <HeatmapGrid data={residual.heatmap_data} />
      </SectionModule>

      {/* Section D: Scatter Plot */}
      <SectionModule
        title="Residual Demand vs Must-Run Floor"
        subtitle="Correlation analysis of grid stability margins"
      >
        <ScatterPlot
          data={residual.hourly_profile}
          correlation={{
            r: residual.correlation_r,
            r2: residual.correlation_r2,
            p: residual.correlation_p,
          }}
        />
      </SectionModule>

      {/* Section E: Curtailment */}
      {curtailment ? (
        <CurtailmentSummary
          ytdGwh={curtailment.ytd_total_gwh}
          forecastGwh={curtailment.ytd_total_gwh * (365 / (new Date().getMonth() * 30 + new Date().getDate() || 1))}
          windGwh={curtailment.ytd_wind_gwh}
          solarGwh={curtailment.ytd_solar_gwh}
        />
      ) : (
        <CurtailmentSummary
          ytdGwh={residual.ytd_curtailment_gwh}
          forecastGwh={residual.forecast_curtailment_gwh}
          windGwh={residual.wind_reduction_gwh}
          solarGwh={residual.solar_reduction_gwh}
        />
      )}

      {/* Section F: Curtailment by Cause */}
      {curtailment && curtailment.hourly_profile.length > 0 && (
        <CurtailmentByCause hourlyProfile={curtailment.hourly_profile} />
      )}

      {/* Daily OZE Curtailment Trend */}
      <HistoricalChart
        endpoint="/api/history/curtailment"
        title="Daily OZE Curtailment"
        yLabel="MWh"
        series={[
          { key: "wi_balance_mwh", label: "Wind Balance", color: "#76d6d5" },
          { key: "wi_network_mwh", label: "Wind Network", color: "#ffb4ab" },
          { key: "pv_balance_mwh", label: "Solar Balance", color: "#ffb692" },
          { key: "pv_network_mwh", label: "Solar Network", color: "#f59e0b" },
        ]}
        defaultDays={30}
      />
    </div>
  );
}
