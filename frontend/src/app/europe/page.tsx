import { getEurope } from "@/lib/api";
import EURankingBar from "@/components/europe/EURankingBar";

export const revalidate = 3600;

export default async function EuropePage() {
  const [result] = await Promise.allSettled([getEurope()]);
  const data = result.status === "fulfilled" ? result.value : null;

  if (!data) {
    return (
      <div className="p-8">
        <div className="bg-surface-container p-6 rounded-xl text-center">
          <span className="material-symbols-outlined text-4xl text-error mb-2">
            error
          </span>
          <h2 className="font-headline text-lg font-bold text-on-surface">
            Unable to load European price data
          </h2>
          <p className="text-sm text-on-surface-variant mt-2">
            Please ensure ENTSO-E API is configured and try refreshing.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="font-headline text-2xl font-bold text-on-surface">
            European DA Price Ranking
          </h1>
          <p className="text-sm text-on-surface-variant mt-1">
            Day-ahead electricity prices across European bidding zones
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span className="bg-surface-container-high px-3 py-1.5 rounded text-on-surface-variant flex items-center gap-2 text-xs">
            <span className="w-2 h-2 rounded-full bg-primary animate-pulse" />
            Live Data Active
          </span>
        </div>
      </div>

      {/* Poland Highlight */}
      <div className="bg-surface-container p-6 rounded-xl">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-4xl text-primary">
              flag
            </span>
            <div>
              <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
                Poland Position
              </p>
              <p className="font-headline text-2xl font-bold text-on-surface">
                #{data.poland_rank}{" "}
                <span className="text-lg text-on-surface-variant font-normal">
                  of {data.rankings.length} zones
                </span>
              </p>
            </div>
          </div>
          <div className="flex items-center gap-8">
            <div className="text-right">
              <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
                PL DA Price
              </p>
              <p className="font-headline text-xl font-bold text-primary">
                €{data.poland_price.toFixed(2)}
              </p>
            </div>
            <div className="text-right">
              <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
                EU Average
              </p>
              <p className="font-headline text-xl font-bold text-on-surface">
                €{data.eu_average.toFixed(2)}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Stats Row */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-surface-container p-5 rounded-xl">
          <div className="flex items-center gap-3">
            <span className="material-symbols-outlined text-error">
              arrow_upward
            </span>
            <div>
              <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
                Most Expensive
              </p>
              <p className="font-headline text-lg font-bold text-on-surface">
                {data.most_expensive.code} — €
                {data.most_expensive.price.toFixed(2)}
              </p>
            </div>
          </div>
        </div>
        <div className="bg-surface-container p-5 rounded-xl">
          <div className="flex items-center gap-3">
            <span className="material-symbols-outlined text-emerald-400">
              arrow_downward
            </span>
            <div>
              <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
                Cheapest
              </p>
              <p className="font-headline text-lg font-bold text-on-surface">
                {data.cheapest.code} — €{data.cheapest.price.toFixed(2)}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Ranking Bars */}
      <div className="bg-surface-container p-6 rounded-xl">
        <h2 className="font-headline text-lg font-bold text-on-surface mb-6">
          Price Ranking by Bidding Zone
        </h2>
        <EURankingBar data={data.rankings} />
      </div>
    </div>
  );
}
