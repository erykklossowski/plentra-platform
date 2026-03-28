import { getReserves, getSpreads } from "@/lib/api";
import LiveBadge from "@/components/ui/LiveBadge";
import SectionModule from "@/components/ui/SectionModule";
import ReservePriceTable from "@/components/reserves/ReservePriceTable";
import ReserveTrendChart from "@/components/reserves/ReserveTrendChart";
import ReserveVsSpreadChart from "@/components/reserves/ReserveVsSpreadChart";

export const revalidate = 3600;

export default async function ReservesPage() {
  const [reservesResult, spreadsResult] = await Promise.allSettled([
    getReserves(),
    getSpreads(),
  ]);
  const reserves =
    reservesResult.status === "fulfilled" ? reservesResult.value : null;
  const spreads =
    spreadsResult.status === "fulfilled" ? spreadsResult.value : null;

  if (!reserves) {
    return (
      <div className="p-8">
        <div className="bg-surface-container p-6 rounded-xl text-center">
          <span className="material-symbols-outlined text-4xl text-error mb-2">
            error
          </span>
          <h2 className="font-headline text-lg font-bold text-on-surface">
            Unable to load reserves data
          </h2>
          <p className="text-sm text-on-surface-variant mt-2">
            PSE CMBP-TP API may be unavailable. Please try refreshing.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-8">
      {/* Page Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="font-headline text-2xl font-bold text-on-surface">
            Reserve Capacity Market
          </h1>
          <p className="text-sm text-on-surface-variant mt-1">
            Balancing reserve prices, trends, and BSP economics
          </p>
        </div>
        <div className="flex items-center gap-3">
          <LiveBadge />
        </div>
      </div>

      {/* Section 1: Current Reserve Prices */}
      <SectionModule
        title="Current Reserve Prices"
        subtitle={`Capacity prices for ${reserves.date} — daily average (PLN/MW)`}
      >
        <ReservePriceTable
          prices={reserves.prices}
          history={reserves.history_13m}
        />
      </SectionModule>

      {/* Section 2: 13-Month Trend */}
      {reserves.history_13m.length > 0 && (
        <SectionModule
          title="13-Month Reserve Price Trend"
          subtitle="Monthly average capacity prices for key upward products"
        >
          <ReserveTrendChart history={reserves.history_13m} />
        </SectionModule>
      )}

      {/* Section 3: Reserve Price vs Clean Spark Spread */}
      {spreads && spreads.history_30d.length > 0 && reserves.daily_30d && reserves.daily_30d.length > 0 && (
        <SectionModule
          title="Reserve Price vs Clean Spark Spread"
          subtitle="When CSS↓, generators seek reserve revenue compensation"
        >
          <ReserveVsSpreadChart
            reserveDaily={reserves.daily_30d}
            spreadHistory={spreads.history_30d}
          />
        </SectionModule>
      )}

      {/* Source attribution */}
      <p className="text-[10px] text-on-surface-variant/60 text-center">
        Source: PSE CMBP-TP · Hourly capacity prices
      </p>
    </div>
  );
}
