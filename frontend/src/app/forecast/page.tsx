import { getForecast } from "@/lib/api";
import DataLoadingCard from "@/components/ui/DataLoadingCard";
import FanChart from "@/components/forecast/FanChart";
import DecompositionChart from "@/components/forecast/DecompositionChart";
import ChangepointAlerts from "@/components/forecast/ChangepointAlerts";

export const dynamic = "force-dynamic";

export default async function ForecastPage() {
  const [result] = await Promise.allSettled([getForecast()]);
  const data = result.status === "fulfilled" ? result.value : null;

  if (!data || data.data_status === "unavailable") {
    return (
      <DataLoadingCard
        section="Forecast"
        message={
          data?.message ??
          "Forecast requires historical data in the database — run backfill first"
        }
      />
    );
  }

  const fuels = data.fuel_forecasts;

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="font-headline text-2xl font-bold text-on-surface">
            Price Forecast
          </h1>
          <p className="text-sm text-on-surface-variant mt-1">
            ETS fuel price forecasts, MSTL decomposition, and structural break
            detection
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span className="bg-surface-container-high px-3 py-1.5 rounded text-on-surface-variant flex items-center gap-2 text-xs">
            <span className="w-2 h-2 rounded-full bg-primary animate-pulse" />
            Updated{" "}
            {data.generated_at
              ? new Date(data.generated_at).toLocaleTimeString("en-GB", {
                  hour: "2-digit",
                  minute: "2-digit",
                })
              : "—"}
          </span>
        </div>
      </div>

      {/* Section 1: Fuel Price Forecasts */}
      <div>
        <h2 className="font-headline text-lg font-bold text-on-surface mb-4">
          14-Day Fuel Price Forecast
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {fuels?.ttf && (
            <FanChart
              ticker="TTF (Natural Gas)"
              lastHistorical={fuels.ttf.last_historical}
              trainingPoints={fuels.ttf.training_points}
              pointForecast={fuels.ttf.point_forecast}
              lower80={fuels.ttf.lower_80}
              upper80={fuels.ttf.upper_80}
              lower95={fuels.ttf.lower_95}
              upper95={fuels.ttf.upper_95}
            />
          )}
          {fuels?.ara && (
            <FanChart
              ticker="ARA (Coal)"
              lastHistorical={fuels.ara.last_historical}
              trainingPoints={fuels.ara.training_points}
              pointForecast={fuels.ara.point_forecast}
              lower80={fuels.ara.lower_80}
              upper80={fuels.ara.upper_80}
              lower95={fuels.ara.lower_95}
              upper95={fuels.ara.upper_95}
            />
          )}
          {fuels?.eua && (
            <FanChart
              ticker="EUA (CO\u2082)"
              lastHistorical={fuels.eua.last_historical}
              trainingPoints={fuels.eua.training_points}
              pointForecast={fuels.eua.point_forecast}
              lower80={fuels.eua.lower_80}
              upper80={fuels.eua.upper_80}
              lower95={fuels.eua.lower_95}
              upper95={fuels.eua.upper_95}
            />
          )}
        </div>
        {!fuels?.ttf && !fuels?.ara && !fuels?.eua && (
          <div className="bg-surface-container p-6 rounded-xl text-center">
            <p className="text-sm text-on-surface-variant">
              Insufficient historical data for forecasting. Run fuel backfill
              (30+ days required).
            </p>
          </div>
        )}
      </div>

      {/* Section 2: TTF Decomposition */}
      {data.decomposition && (
        <div>
          <h2 className="font-headline text-lg font-bold text-on-surface mb-4">
            TTF Price Decomposition
          </h2>
          <DecompositionChart
            trend={data.decomposition.trend}
            seasonal7d={data.decomposition.seasonal_7d}
            residual={data.decomposition.residual}
          />
        </div>
      )}

      {/* Section 3: Changepoint Alerts */}
      <div>
        <h2 className="font-headline text-lg font-bold text-on-surface mb-3">
          Structural Break Detection
        </h2>
        <ChangepointAlerts alert={data.changepoint_alerts ?? null} />
      </div>

    </div>
  );
}
