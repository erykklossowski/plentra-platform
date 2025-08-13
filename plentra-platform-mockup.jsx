import React, { useMemo, useState, useEffect } from "react";
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  BarChart,
  Bar,
  ComposedChart,
  Scatter,
  RadialBarChart,
  RadialBar,
  ReferenceLine,
} from "recharts";
import { Menu, Bell, Settings, ChevronDown, TrendingUp, AlertTriangle, Sun, Moon, Download, Share2, SlidersHorizontal, X, CheckCircle2, Wifi, WifiOff } from "lucide-react";

// =============================
// ENERGY PRICE OBSERVABILITY & PREDICTABILITY – C&I WEB APP MOCKUP
// Single-file React mockup with Tailwind + Recharts
// Design system, widgets, interactions, responsive behavior, and technical annotations included inline.
// =============================

// ---------- THEME & DESIGN TOKENS ----------
const COLORS = {
  primary: "#1e3a8a", // Deep blue
  secondary: "#0891b2", // Teal
  accent: "#f59e0b", // Amber
  success: "#10b981", // Green
  danger: "#ef4444", // Red
  bg: "#f9fafb", // Light gray
  card: "#ffffff",
  darkBg: "#0f172a", // Slate-900
};

// ---------- UTILITIES ----------
const formatPrice = (v) => `€${Number(v).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}/MWh`;
const formatNum = (v) => Number(v).toLocaleString();
const tsFmt = (d) => new Date(d).toLocaleString(undefined, { month: 'short', day: '2-digit', hour: '2-digit', minute: '2-digit' });

// Mock data generators for charts (for visualization only)
function useMockTimeseries(points = 24, start = Date.now() - 23*3600*1000, stepMs = 3600*1000, base = 60, vol = 20) {
  return useMemo(() => Array.from({ length: points }, (_, i) => {
    const t = start + i*stepMs;
    const v = base + Math.sin(i/3)*vol + (Math.random()-0.5)*vol*0.5;
    return { ts: t, value: Math.max(0, Math.round(v*100)/100) };
  }), [points, start, stepMs, base, vol]);
}

function classNames(...a){ return a.filter(Boolean).join(' '); }

// ---------- LAYOUT SHELL ----------
export default function App() {
  const [route, setRoute] = useState("dashboard");
  const [dark, setDark] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [showAlert, setShowAlert] = useState(true);
  const [connectionOk, setConnectionOk] = useState(true);
  const [loading, setLoading] = useState(false);
  const [empty, setEmpty] = useState(false);
  const [error, setError] = useState(false);

  // Simulate live updates micro-interaction
  const [lastUpdated, setLastUpdated] = useState(Date.now());
  useEffect(() => {
    const id = setInterval(() => setLastUpdated(Date.now()), 15000);
    return () => clearInterval(id);
  }, []);

  // Responsive flags (mock)
  const isMobile = typeof window !== 'undefined' ? window.innerWidth < 768 : false;

  return (
    <div className={classNames("min-h-screen", dark ? "dark" : "")}>
      <div className={classNames("min-h-screen transition-colors", dark ? "bg-slate-900 text-slate-100" : "text-slate-900")} style={!dark ? { backgroundColor: COLORS.bg } : {}}>        
        <TopNav
          dark={dark}
          setDark={setDark}
          route={route}
          setRoute={setRoute}
          spotPrice={47.82}
          connectionOk={connectionOk}
        />

        {showAlert && (
          <AlertBanner onClose={() => setShowAlert(false)} />
        )}

        <div className="flex">
          <Sidebar open={sidebarOpen} setOpen={setSidebarOpen} />

          <main className="flex-1 p-4 md:p-6 lg:p-8">
            {/* Data freshness / performance indicators */}
            <div className="flex items-center gap-3 mb-4">
              <span className="text-xs opacity-70">Last updated: {new Date(lastUpdated).toLocaleTimeString()}</span>
              <span className={classNames("text-xs px-2 py-0.5 rounded-full", connectionOk ? "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300" : "bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300")}>{connectionOk ? "Live" : "Offline"}</span>
              <span className="text-xs px-2 py-0.5 rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300">Data freshness: <b>15s</b></span>
              <div className="flex-1" />
              <button className="text-xs underline opacity-70" onClick={()=>setLoading(!loading)}>Toggle Loading</button>
              <button className="text-xs underline opacity-70 ml-2" onClick={()=>setEmpty(!empty)}>Toggle Empty</button>
              <button className="text-xs underline opacity-70 ml-2" onClick={()=>setError(!error)}>Toggle Error</button>
            </div>

            {route === "dashboard" && (
              <DashboardPage loading={loading} empty={empty} error={error} />
            )}
            {route === "short" && <ShortTermPage />}
            {route === "mid" && <MidTermPage />}
            {route === "long" && <LongTermPage />}
            {route === "alerts" && <AlertsPage />}
            {route === "reports" && <ReportsPage />}
          </main>
        </div>

        <FloatingActions />
        <Footer />
      </div>
    </div>
  );
}

// ---------- TOP NAVIGATION (Fixed, 64px) ----------
function TopNav({ dark, setDark, route, setRoute, spotPrice, connectionOk }){
  return (
    <header className="sticky top-0 z-40 bg-white/90 backdrop-blur border-b dark:bg-slate-900/80 dark:border-slate-800 h-16">
      <div className="h-full px-4 md:px-6 lg:px-8 flex items-center gap-4">
        <button className="md:hidden p-2 rounded hover:bg-slate-100 dark:hover:bg-slate-800"><Menu size={18}/></button>
        <div className="flex items-center gap-2 font-semibold text-[color:var(--color-primary)]" style={{ ['--color-primary']: COLORS.primary }}>
          <div className="w-8 h-8 rounded" style={{ backgroundColor: COLORS.primary }} />
          <span>GridLens</span>
        </div>

        <nav className="hidden md:flex items-center gap-2 ml-4">
          {[
            {k:"dashboard", label:"Dashboard"},
            {k:"short", label:"Short-Term"},
            {k:"mid", label:"Mid-Term"},
            {k:"long", label:"Long-Term"},
            {k:"alerts", label:"Alerts"},
            {k:"reports", label:"Reports"},
          ].map(i => (
            <button key={i.k} onClick={()=>setRoute(i.k)} className={classNames("px-3 py-2 rounded-md text-sm font-medium transition", route===i.k ? "bg-slate-100 text-slate-900 dark:bg-slate-800 dark:text-slate-100" : "hover:bg-slate-100 dark:hover:bg-slate-800")}>{i.label}</button>
          ))}
        </nav>

        {/* Center ticker */}
        <div className="mx-auto hidden md:flex items-center gap-2 text-sm">
          <span className="opacity-70">Current Spot</span>
          <span className="font-mono tabular-nums font-semibold">{formatPrice(spotPrice)}</span>
          <TrendingUp className="text-emerald-500" size={16}/>
        </div>

        <div className="flex-1" />

        <div className="hidden md:flex items-center gap-3">
          <button className="p-2 rounded hover:bg-slate-100 dark:hover:bg-slate-800"><Bell size={18} /></button>
          <button className="p-2 rounded hover:bg-slate-100 dark:hover:bg-slate-800"><Settings size={18} /></button>
          <div className="w-px h-6 bg-slate-200 dark:bg-slate-700" />
          <button className="px-3 py-1.5 rounded-md text-sm text-white hover:opacity-90" style={{ backgroundColor: COLORS.secondary }}>New Alert</button>
          <button onClick={()=>setDark(!dark)} className="p-2 rounded hover:bg-slate-100 dark:hover:bg-slate-800" aria-label="Toggle dark mode">
            {dark ? <Sun size={18}/> : <Moon size={18}/>}
          </button>
          <div className="flex items-center gap-2 text-xs px-2 py-1 rounded-full border dark:border-slate-700">
            {connectionOk ? <Wifi size={14} className="text-emerald-500"/> : <WifiOff size={14} className="text-red-500"/>}
            <span>{connectionOk ? "Connected" : "Disconnected"}</span>
          </div>
          <div className="flex items-center gap-2 ml-2">
            <div className="w-8 h-8 rounded-full bg-slate-300" />
            <div className="text-sm">Alex Morgan</div>
            <ChevronDown size={14} />
          </div>
        </div>
      </div>
    </header>
  );
}

// ---------- LEFT SIDEBAR (Collapsible) ----------
function Sidebar({ open, setOpen }){
  return (
    <aside className={classNames("transition-all border-r dark:border-slate-800 bg-white dark:bg-slate-900", open ? "w-60" : "w-14")}>      
      <div className="p-3 flex items-center justify-between">
        <span className={classNames("text-sm font-medium", !open && "sr-only")}>Filters</span>
        <button onClick={()=>setOpen(!open)} className="p-2 rounded hover:bg-slate-100 dark:hover:bg-slate-800"><Menu size={16}/></button>
      </div>
      <div className="p-3 space-y-4">
        <div>
          <label className={classNames("text-xs block mb-1 opacity-70", !open && "sr-only")}>
            Market
          </label>
          <div className="flex gap-2">
            <select className="w-full px-2 py-2 rounded border dark:border-slate-700 bg-white dark:bg-slate-800 text-sm">
              <option>PL (POLPX)</option>
              <option>DE (EPEX)</option>
              <option>FR (EPEX)</option>
              <option>Nordics (NO/SE)</option>
            </select>
          </div>
        </div>
        <div>
          <label className={classNames("text-xs block mb-1 opacity-70", !open && "sr-only")}>Time Range</label>
          <div className="grid grid-cols-2 gap-2">
            {['24h','7d','30d','YTD'].map(t => (
              <button key={t} className="px-2 py-1.5 rounded border text-xs hover:bg-slate-50 dark:hover:bg-slate-800 dark:border-slate-700">{t}</button>
            ))}
          </div>
        </div>
        <div>
          <label className={classNames("text-xs block mb-1 opacity-70", !open && "sr-only")}>Saved Views</label>
          <ul className="space-y-1 text-sm">
            {['C&I Poland','Intraday Watch','mFRR Focus','Long Hedging'].map(n => (
              <li key={n} className="px-2 py-1.5 rounded hover:bg-slate-50 dark:hover:bg-slate-800 cursor-pointer">{open ? n : n.slice(0,2)}</li>
            ))}
          </ul>
        </div>
        <div>
          <label className={classNames("text-xs block mb-1 opacity-70", !open && "sr-only")}>Region / Zone</label>
          <select className="w-full px-2 py-2 rounded border dark:border-slate-700 bg-white dark:bg-slate-800 text-sm">
            <option>PL-CEN</option>
            <option>PL-W</option>
            <option>PL-E</option>
            <option>PL-N</option>
          </select>
        </div>
      </div>
    </aside>
  );
}

// ---------- ALERT BANNER ----------
function AlertBanner({ onClose }){
  return (
    <div className="fixed top-16 inset-x-0 z-30 max-w-5xl mx-auto">
      <div className="mx-4 md:mx-0 bg-amber-50 dark:bg-amber-900/30 border border-amber-200 dark:border-amber-800 text-amber-900 dark:text-amber-200 rounded-md shadow p-3 flex items-start gap-3 animate-slide-in">
        <AlertTriangle className="mt-0.5"/>
        <div className="text-sm">
          <b>Reserve spike detected:</b> aFRR prices exceeded 300 €/MWh in DE between 12:00–13:00. <button className="underline">See details</button>
        </div>
        <button onClick={onClose} className="ml-auto p-1 rounded hover:bg-amber-100/60 dark:hover:bg-amber-800/40"><X size={16}/></button>
      </div>
    </div>
  );
}

// ---------- DASHBOARD PAGE (Main) ----------
function DashboardPage({ loading, empty, error }){
  // Metric cards data
  const weekly = useMockTimeseries(7, Date.now()-6*24*3600*1000, 24*3600*1000, 62, 15);
  const monthly = useMockTimeseries(30, Date.now()-29*24*3600*1000, 24*3600*1000, 58, 18);

  return (
    <div className="space-y-6">
      {/* Header metric cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <MetricCard title="Current Spot Price" value={formatPrice(47.82)} delta={+2.4} loading={loading} spark={useMockTimeseries(20)} />
        <MetricCard title="Day-Ahead Average (Tomorrow)" value={formatPrice(55.10)} delta={-1.1} loading={loading} spark={useMockTimeseries(24)} />
        <MetricCard title="Weekly Forecast" value="7-day trend" extra={`${formatPrice(Math.min(...weekly.map(d=>d.value)))} – ${formatPrice(Math.max(...weekly.map(d=>d.value)))}`} loading={loading} spark={weekly} />
        <MetricCard title="Monthly Outlook" value="Volatility Index 0.62" extra="30-day trajectory" loading={loading} spark={monthly} />
      </div>

      {/* Primary visualizations */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Real-Time Grid Balance" subtitle="Last 24h – Generation mix vs demand" loading={loading} error={error} empty={empty}
          annotation={`API: /api/grid/balance?market=PL&lookback=24h\nUpdate: 15s WS stream\nNotes: Stacked area by fuel; demand overlay. Colors fixed per legend. Source: TSO transparency.`}
        >
          <RealTimeGridBalance/>
        </Card>

        <Card title="Intraday Price Curve" subtitle="Hourly OHLC with volume & VWAP" loading={loading} error={error} empty={empty}
          annotation={`API: /api/price/intraday_ohlc?market=PL&date=today\nUpdate: 60s\nCalc: VWAP=Σ(p_i*v_i)/Σ(v_i), OHLC aggregated hourly. Source: Exchange intraday tape. Caching: 1h immutable.`}
        >
          <IntradayCandles/>
        </Card>
      </div>

      {/* Market Dynamics */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        <Card title="Reserve Market" subtitle="FCR / aFRR / mFRR" annotation={`API: /api/reserve/prices?market=PL&range=7d\nUpdate: 5m\nSparklines: last 7d. Thresholds: Green<100, Yellow 100–200, Red>200 €/MWh.`}>
          <ReserveGauges/>
        </Card>
        <Card title="Cross-Border Flows" subtitle="Animated net flows & price gradient" annotation={`API: /api/flows/net?market=PL&horizon=rt\nUpdate: 5m\nSources: ENTSO-E, XBID. Colors: green=cheap -> red=expensive. Arrows scaled to MW.`}>
          <CrossBorderMap/>
        </Card>
        <Card title="Merit Order Curve" subtitle="Generation stack & marginal cost" annotation={`API: /api/merit_order?market=PL&ts=now\nUpdate: hourly\nCalc: Sort units by SRMC, cumulative capacity on X. Demand as vertical line. Shaded area = dispatched set.`}>
          <MeritOrder/>
        </Card>
      </div>

      {/* Advanced Features */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <Card title="Price Heatmap" subtitle="Next 7 days by hour" annotation={`API: /api/price/dayahead?market=PL&horizon=7d\nUpdate: daily (D-1 12:00) + intraday refresh\nCache: 7d sliding window in CDN`}>          
          <PriceHeatmap/>
        </Card>
        <Card title="Weather Impact" subtitle="Wind, solar, temp deviation, precipitation" annotation={`APIs: /api/weather/wind_map, /solar/irradiance, /temp/deviation, /precip/outlook\nUpdate: 1h\nNotes: ECMWF ensemble P50. Attribution: ECMWF, Meteostat.`}>
          <WeatherImpact/>
        </Card>
      </div>

      <Card title="Forward Curve" subtitle="Monthly forwards & historical envelope" annotation={`API: /api/forwards/monthly?market=PL&horizon=5y\nUpdate: EOD\nCalc: 10th–90th pct envelope from last 5y; current forward bold; hist avg dashed.`}>
        <ForwardCurve/>
      </Card>

      {/* Data Table */}
      <Card title="Recent Trades" subtitle="Most recent intraday executions" annotation={`API: /api/trades/recent?market=PL&limit=50\nUpdate: 10s\nExport: CSV/Excel. Sorting client-side unless server-sorted.`}>
        <TradesTable/>
      </Card>
    </div>
  );
}

// ---------- SHORT-TERM PAGE (Intraday focus) ----------
function ShortTermPage(){
  return (
    <div className="space-y-6">
      <SectionTitle title="Short-Term Analysis" subtitle="Intraday focus: 0–48h"/>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Live Imbalance Direction" subtitle="Under/Over-contracted signal" annotation={`API: /api/imbalance/signal?market=PL&window=5m\nUpdate: 5s\nCalc: sign(NIV).`}>          
          <LiveImbalance/>
        </Card>
        <Card title="Spread Radar" subtitle="Min–Max price spread (hourly)" annotation={`API: /api/price/spread?market=PL&lookback=24h\nUpdate: 1m\nCalc: max(price)-min(price).`}>
          <SpreadChart/>
        </Card>
      </div>
      <Card title="Battery Dispatch Helper" subtitle="Arb hints (SoC-aware)" annotation={`API: /api/advice/arb?assetId=BESS-123&horizon=24h\nUpdate: 5m\nCalc: RL agent constrained by SoC/SoH; risk-adjusted.`}>
        <DispatchHelper/>
      </Card>
    </div>
  );
}

// ---------- MID-TERM PAGE (Weekly patterns) ----------
function MidTermPage(){
  return (
    <div className="space-y-6">
      <SectionTitle title="Mid-Term Analysis" subtitle="Weekly patterns & seasonality"/>
      <Card title="Weekly Seasonality" subtitle="Median profile by weekday" annotation={`API: /api/price/seasonality?market=PL&bucket=hour,weekday\nUpdate: daily\nCalc: median by hour-of-day & weekday.`}>
        <WeeklySeasonality/>
      </Card>
      <Card title="Reserve Price Trends" subtitle="FCR/aFRR/mFRR 12 weeks" annotation={`API: /api/reserve/trends?market=PL&weeks=12\nUpdate: daily EOD`}>
        <ReserveTrends/>
      </Card>
    </div>
  );
}

// ---------- LONG-TERM PAGE (Scenarios) ----------
function LongTermPage(){
  return (
    <div className="space-y-6">
      <SectionTitle title="Long-Term View" subtitle="5-year scenarios & macro drivers"/>
      <Card title="Scenario Planning" subtitle="Base / High RES / Gas Shock" annotation={`API: /api/scenarios?market=PL&horizon=5y\nUpdate: on-demand\nCalc: stochastic paths w/ Monte Carlo on fuel/CO2; RES build-out per policy.`}>
        <ScenarioPlanner/>
      </Card>
      <Card title="Fundamentals" subtitle="Fuel, CO₂, load growth assumptions" annotation={`APIs: /api/fuels (NG, coal), /api/co2, /api/load_forecast\nUpdate: weekly/EOD as applicable`}>
        <Fundamentals/>
      </Card>
    </div>
  );
}

// ---------- ALERTS PAGE (Config + Modal) ----------
function AlertsPage(){
  const [open, setOpen] = useState(false);
  return (
    <div className="space-y-6">
      <SectionTitle title="Alerts" subtitle="Manage thresholds & routing"/>
      <Card title="Active Alerts" subtitle="Rules & status" annotation={`API: /api/alerts/list\nUpdate: 30s`}>
        <AlertsTable onCreate={()=>setOpen(true)}/>
      </Card>
      {open && <AlertModal onClose={()=>setOpen(false)} />}
    </div>
  );
}

function ReportsPage(){
  return (
    <div className="space-y-6">
      <SectionTitle title="Reports" subtitle="Export, share, and schedule"/>
      <Card title="Monthly Risk Report" subtitle="Volatility, VaR, P&L attribution" annotation={`API: /api/reports/run?type=monthly\nUpdate: on-demand\nCache: persisted for 90d`}>
        <div className="p-6 text-sm">Choose a template and generate a PDF. <button className="px-3 py-1.5 rounded bg-slate-900 text-white dark:bg-white dark:text-slate-900">Generate</button></div>
      </Card>
    </div>
  );
}

// ---------- REUSABLE UI ----------
function SectionTitle({ title, subtitle }){
  return (
    <div className="flex items-end justify-between">
      <div>
        <h1 className="text-xl md:text-2xl font-semibold">{title}</h1>
        <p className="text-sm opacity-70">{subtitle}</p>
      </div>
      <div className="flex items-center gap-2">
        <button className="px-3 py-1.5 rounded-md border text-sm hover:bg-slate-50 dark:hover:bg-slate-800 dark:border-slate-700"><Download size={16} className="inline mr-1"/> Export</button>
        <button className="px-3 py-1.5 rounded-md border text-sm hover:bg-slate-50 dark:hover:bg-slate-800 dark:border-slate-700"><Share2 size={16} className="inline mr-1"/> Share</button>
      </div>
    </div>
  );
}

function Card({ title, subtitle, children, annotation, loading=false, empty=false, error=false }){
  return (
    <div className="bg-white dark:bg-slate-900 rounded-lg shadow-sm border border-slate-200 dark:border-slate-800">
      <div className="p-4 border-b dark:border-slate-800">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="font-semibold">{title}</h3>
            {subtitle && <p className="text-xs opacity-70">{subtitle}</p>}
          </div>
          <div className="text-[10px] opacity-60 text-right whitespace-pre-line hidden md:block">{annotation}</div>
        </div>
      </div>
      <div className="p-3 md:p-4">
        {loading ? <Skeleton height={220}/> : error ? <ErrorState/> : empty ? <EmptyState/> : children}
      </div>
    </div>
  );
}

function Skeleton({ height=180 }){
  return (
    <div className="animate-pulse space-y-3">
      <div className="h-4 bg-slate-200 dark:bg-slate-800 rounded"/>
      <div className="h-4 w-5/6 bg-slate-200 dark:bg-slate-800 rounded"/>
      <div className="h-[1px] bg-slate-200 dark:bg-slate-800"/>
      <div className="bg-slate-200 dark:bg-slate-800 rounded" style={{ height: height }}/>
    </div>
  );
}

function EmptyState(){
  return (
    <div className="p-6 text-center">
      <div className="mx-auto w-20 h-20 rounded-full bg-slate-100 dark:bg-slate-800 flex items-center justify-center mb-3">📊</div>
      <div className="font-medium">No data yet</div>
      <div className="text-sm opacity-70">Connect a data source or adjust filters to populate this widget.</div>
      <button className="mt-3 px-3 py-1.5 rounded text-white" style={{ backgroundColor: COLORS.secondary }}>Connect Source</button>
    </div>
  );
}

function ErrorState(){
  return (
    <div className="p-6 text-center">
      <div className="mx-auto w-20 h-20 rounded-full bg-red-50 dark:bg-red-900/40 flex items-center justify-center mb-3 text-red-600">⚠️</div>
      <div className="font-medium">We hit a snag</div>
      <div className="text-sm opacity-70">Couldn’t load this data. Retrying may help. Falling back to cached.</div>
      <div className="mt-2 text-xs opacity-60">Using cached snapshot from 12:05</div>
      <button className="mt-3 px-3 py-1.5 rounded border text-sm hover:bg-slate-50 dark:hover:bg-slate-800">Retry</button>
    </div>
  );
}

function MetricCard({ title, value, delta, extra, spark, loading }){
  return (
    <div className="bg-white dark:bg-slate-900 rounded-lg shadow-sm border border-slate-200 dark:border-slate-800 p-4">
      {loading ? <Skeleton height={60}/> : (
        <>
          <div className="text-xs opacity-70">{title}</div>
          <div className="flex items-end justify-between mt-1">
            <div>
              <div className="text-2xl font-semibold font-mono tabular-nums">{value}</div>
              {extra && <div className="text-xs opacity-70">{extra}</div>}
            </div>
            {typeof delta === 'number' && (
              <div className={classNames("text-xs px-2 py-0.5 rounded-full", delta>=0 ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300" : "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300")}>{delta>=0?'+':''}{delta}%</div>
            )}
          </div>
          <div className="h-14 mt-2">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={spark} margin={{ top: 4, right: 0, left: 0, bottom: 0 }}>
                <defs>
                  <linearGradient id="grad" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0%" stopColor={COLORS.secondary} stopOpacity={0.4} />
                    <stop offset="100%" stopColor={COLORS.secondary} stopOpacity={0.05} />
                  </linearGradient>
                </defs>
                <Area type="monotone" dataKey="value" stroke={COLORS.secondary} fill="url(#grad)" strokeWidth={2} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </>
      )}
    </div>
  );
}

// ---------- CHARTS & WIDGETS ----------
function RealTimeGridBalance(){
  // Build layered areas for generation mix + demand line
  const data = useMemo(() => {
    const base = useMockTimeseries(24, Date.now()-23*3600*1000, 3600*1000, 60, 15);
    return base.map((d,i)=>({
      ts: d.ts,
      nuclear: 12+Math.sin(i/8)*1,
      gas: 15+Math.cos(i/4)*3,
      coal: 18+Math.sin(i/5)*2,
      wind: 7+Math.max(0,Math.sin(i/2))*6,
      solar: Math.max(0,(Math.sin((i-6)/3)+1)*5),
      hydro: 5+Math.cos(i/6)*1.2,
      demand: 62 + Math.sin(i/3)*8
    }));
  }, []);

  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="ts" tickFormatter={tsFmt} interval={3} />
          <YAxis />
          <Tooltip formatter={(v, n)=>[formatNum(v)+" GW", n]} labelFormatter={(l)=>tsFmt(l)} />
          <Legend verticalAlign="bottom" height={24} />
          <Area stackId="1" type="monotone" dataKey="nuclear" stroke="#7e22ce" fill="#7e22ce22" name="Nuclear" />
          <Area stackId="1" type="monotone" dataKey="gas" stroke="#ea580c" fill="#ea580c22" name="Gas" />
          <Area stackId="1" type="monotone" dataKey="coal" stroke="#6b7280" fill="#6b728022" name="Coal" />
          <Area stackId="1" type="monotone" dataKey="wind" stroke="#60a5fa" fill="#60a5fa33" name="Wind" />
          <Area stackId="1" type="monotone" dataKey="solar" stroke="#facc15" fill="#facc1533" name="Solar" />
          <Area stackId="1" type="monotone" dataKey="hydro" stroke="#1e3a8a" fill="#1e3a8a22" name="Hydro" />
          <Line type="monotone" dataKey="demand" stroke="#000000" strokeWidth={2} dot={false} name="Demand" />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

function IntradayCandles(){
  // Generate OHLC mock data hourly
  const hours = Array.from({length: 24}, (_,i)=>i);
  const data = hours.map(h=>{
    const open = 40 + Math.sin(h/3)*8 + Math.random()*2;
    const close = open + (Math.random()-0.5)*6;
    const high = Math.max(open, close) + Math.random()*5 + 2;
    const low = Math.min(open, close) - Math.random()*5 - 2;
    const vol = 50 + Math.random()*200;
    const vwap = (open+close+high+low)/4 + (Math.random()-0.5)*2;
    return { hour: h, open, high, low, close, vol, vwap };
  });

  // Custom candlestick using Recharts Customized via Bar ranges
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="hour" tickFormatter={(h)=>`${String(h).padStart(2,'0')}:00`} />
          <YAxis tickFormatter={(v)=>`€${v}`} />
          <Tooltip formatter={(v,n)=> n==='vol' ? [formatNum(v), 'Volume'] : [formatPrice(v),''] } labelFormatter={(l)=>`Hour ${l}:00`} />
          <Legend verticalAlign="bottom" height={24} />
          {/* Candle wicks */}
          <Bar dataKey={(d)=>d.high-d.low} name="Range" barSize={4} stackId="range" fill="#94a3b8" shape={props=>{
            const { x, y, width, height, payload } = props;
            const midX = x + width/2;
            const yHigh = props.y;
            const yLow = y + height;
            return (
              <g>
                <line x1={midX} x2={midX} y1={yHigh} y2={yLow} stroke="#64748b" strokeWidth={1} />
                {/* Body */}
                {(() => {
                  const yOpen = props.y + (props.height * (props.payload.high - props.payload.open)) / (props.payload.high - props.payload.low);
                  const yClose = props.y + (props.height * (props.payload.high - props.payload.close)) / (props.payload.high - props.payload.low);
                  const top = Math.min(yOpen, yClose);
                  const bodyH = Math.max(2, Math.abs(yOpen - yClose));
                  const green = props.payload.close >= props.payload.open;
                  return <rect x={midX-6} y={top} width={12} height={bodyH} fill={green?"#10b981":"#ef4444"} rx={2}/>;
                })()}
              </g>
            );
          }} />
          {/* VWAP */}
          <Line type="monotone" dataKey="vwap" stroke="#3b82f6" strokeDasharray="4 4" name="VWAP" dot={false}/>
          {/* Volume */}
          <Bar dataKey="vol" yAxisId="right" fill="#e5e7eb" name="Volume" />
          <YAxis yAxisId="right" orientation="right" hide />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

function ReserveGauges(){
  const items = [
    { name: 'FCR', val: 85 },
    { name: 'aFRR', val: 210 },
    { name: 'mFRR', val: 130 },
  ];
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {items.map(it => (
        <div key={it.name} className="p-3 rounded border dark:border-slate-800">
          <div className="text-xs opacity-70 mb-2">{it.name}</div>
          <div className="h-40">
            <ResponsiveContainer width="100%" height="100%">
              <RadialBarChart innerRadius="70%" outerRadius="100%" data={[{ name: it.name, uv: it.val }]} startAngle={180} endAngle={0}>
                <RadialBar minAngle={15} background clockWise dataKey="uv" fill={it.val<100?"#10b981": it.val>200?"#ef4444":"#f59e0b"} />
                <text x="50%" y="60%" textAnchor="middle" dominantBaseline="middle" className="fill-current" style={{ fontSize: 18, fontFamily:'monospace' }}>{formatNum(it.val)}</text>
                <text x="50%" y="75%" textAnchor="middle" dominantBaseline="middle" className="fill-current" style={{ fontSize: 10 }}>{"€/MWh"}</text>
              </RadialBarChart>
            </ResponsiveContainer>
          </div>
          <div className="mt-2 h-8">
            <ResponsiveContainer width="100%" height="100%">
              <Sparkline />
            </ResponsiveContainer>
          </div>
        </div>
      ))}
    </div>
  );
}

function Sparkline(){
  const d = useMockTimeseries(14, Date.now()-7*24*3600*1000, 12*3600*1000, 150, 40);
  return (
    <AreaChart data={d}>
      <Area type="monotone" dataKey="value" stroke="#64748b" fill="#cbd5e133" strokeWidth={1}/>
    </AreaChart>
  );
}

function CrossBorderMap(){
  // Simplified SVG map mock with arrows & labels
  const arrows = [
    { from: 'DE', to: 'PL', x1: 40, y1: 70, x2: 110, y2: 65, flow: 1200, priceFrom: 65, priceTo: 58 },
    { from: 'CZ', to: 'PL', x1: 90, y1: 100, x2: 110, y2: 80, flow: 600, priceFrom: 52, priceTo: 58 },
  ];
  return (
    <div className="h-72 p-2">
      <svg viewBox="0 0 200 140" className="w-full h-full rounded bg-slate-50 dark:bg-slate-800">
        {/* Countries (very abstract) */}
        <rect x="100" y="50" width="60" height="35" fill="#e2e8f0" rx="2"/> {/* PL */}
        <rect x="40" y="55" width="55" height="35" fill="#e2e8f0" rx="2"/> {/* DE */}
        <rect x="85" y="95" width="30" height="30" fill="#e2e8f0" rx="2"/> {/* CZ */}
        {/* Labels with prices */}
        <text x="130" y="68" fontSize="6" textAnchor="middle">PL €58</text>
        <text x="68" y="73" fontSize="6" textAnchor="middle">DE €65</text>
        <text x="100" y="110" fontSize="6" textAnchor="middle">CZ €52</text>
        {/* Arrows */}
        {arrows.map((a,i)=>{
          const thickness = Math.min(6, Math.max(2, a.flow/400));
          const priceDiff = Math.max(0, Math.min(1, (a.priceFrom - a.priceTo + 20)/40));
          const r = Math.floor(255*priceDiff);
          const g = Math.floor(255*(1-priceDiff));
          const color = `rgb(${r},${g},0)`; // green->red
          return (
            <g key={i}>
              <line x1={a.x1} y1={a.y1} x2={a.x2} y2={a.y2} stroke={color} strokeWidth={thickness} markerEnd="url(#arrow)"/>
              <text x={(a.x1+a.x2)/2} y={(a.y1+a.y2)/2 - 3} fontSize="5" textAnchor="middle">{a.flow} MW</text>
            </g>
          );
        })}
        <defs>
          <marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="currentColor" />
          </marker>
        </defs>
      </svg>
    </div>
  );
}

function MeritOrder(){
  const stack = Array.from({length: 12}, (_,i)=>({ cap: (i+1)*2, cost: 30 + i*6 + (Math.random()*4) }));
  const demand = 17; // GW
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={stack} margin={{ top: 10, right: 10, left: 10, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="cap" label={{ value: 'Cumulative capacity (GW)', position: 'insideBottom', offset: -5 }} />
          <YAxis label={{ value: 'Marginal cost (€/MWh)', angle: -90, position: 'insideLeft' }} />
          <Tooltip formatter={(v,n)=> n==='cost'?formatPrice(v):formatNum(v)} />
          <Legend verticalAlign="bottom" height={24} />
          <StepLine dataKey="cost" />
          <ReferenceLine x={demand} stroke="#111827" label={{ value: 'Demand', position: 'top' }} />
          {/* Shaded area under active generators */}
          <Area type="step" dataKey="cost" stroke="#93c5fd" fill="#93c5fd33" name="Active generators" />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

function StepLine({ dataKey }){
  // Renders a step-like curve using a Line with monotoneStep not available; simulate with type="linear" and appropriate data spacing
  return <Line type="step" dataKey={dataKey} stroke="#0ea5e9" dot={false} name="Merit order" />
}

function PriceHeatmap(){
  const days = 7;
  const hours = 24;
  const data = Array.from({length: days}, (_,d)=> Array.from({length: hours}, (_,h)=> 20 + 50*Math.random() + 30*Math.sin((d+h)/4)));
  const max = Math.max(...data.flat());
  const min = Math.min(...data.flat());
  const cell = (v)=>{
    const t = (v - min) / (max - min + 0.0001);
    const r = Math.round(255 * t);
    const b = Math.round(255 * (1-t));
    return `rgb(${r},${Math.round(80*(1-t))},${b})`;
  };
  return (
    <div className="overflow-auto">
      <div className="grid" style={{ gridTemplateColumns: `80px repeat(${days}, minmax(60px, 1fr))` }}>
        <div></div>
        {Array.from({length: days}, (_,d)=> <div key={d} className="text-xs text-center py-1 opacity-70">{new Date(Date.now()+d*24*3600*1000).toLocaleDateString(undefined,{weekday:'short', month:'short', day:'2-digit'})}</div>)}
        {Array.from({length: hours}, (_,h)=> (
          <React.Fragment key={h}>
            <div className="text-xs sticky left-0 bg-white dark:bg-slate-900 py-1 pr-2">{String(h).padStart(2,'0')}:00</div>
            {Array.from({length: days}, (_,d)=> (
              <div key={d} className="h-8 flex items-center justify-center text-[10px] font-mono tabular-nums" style={{ backgroundColor: cell(data[d][h]) }}>{Math.round(data[d][h])}</div>
            ))}
          </React.Fragment>
        ))}
      </div>
    </div>
  );
}

function WeatherImpact(){
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
      <MiniCard title="Wind forecast map">
        <div className="h-40 rounded bg-gradient-to-br from-sky-200 to-sky-400 dark:from-sky-900 dark:to-sky-700 flex items-center justify-center text-sm opacity-80">Map placeholder (wind speeds)</div>
      </MiniCard>
      <MiniCard title="Solar irradiance">
        <div className="h-40">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={useMockTimeseries(48, Date.now(), 3600*1000, 3, 3)}>
              <Area dataKey="value" stroke="#f59e0b" fill="#f59e0b33" />
              <CartesianGrid stroke="#e5e7eb" />
              <XAxis dataKey="ts" tickFormatter={tsFmt} interval={7} />
              <YAxis />
              <Tooltip labelFormatter={(l)=>tsFmt(l)} />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </MiniCard>
      <MiniCard title="Temperature deviation">
        <div className="h-40">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={useMockTimeseries(72, Date.now(), 3600*1000, 0, 5)}>
              <Line dataKey="value" stroke="#ef4444" dot={false} />
              <CartesianGrid stroke="#e5e7eb" />
              <XAxis dataKey="ts" tickFormatter={tsFmt} interval={9} />
              <YAxis />
              <Tooltip labelFormatter={(l)=>tsFmt(l)} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </MiniCard>
      <MiniCard title="Precipitation outlook">
        <div className="h-40">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={useMockTimeseries(72, Date.now(), 3600*1000, 2, 2)}>
              <Bar dataKey="value" fill="#60a5fa" />
              <CartesianGrid stroke="#e5e7eb" />
              <XAxis dataKey="ts" tickFormatter={tsFmt} interval={9} />
              <YAxis />
              <Tooltip labelFormatter={(l)=>tsFmt(l)} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </MiniCard>
    </div>
  );
}

function MiniCard({ title, children }){
  return (
    <div className="p-3 rounded border dark:border-slate-800">
      <div className="text-xs opacity-70 mb-2">{title}</div>
      {children}
    </div>
  );
}

function ForwardCurve(){
  const data = Array.from({length: 36}, (_,i)=>({
    m: i,
    current: 60 + Math.sin(i/6)*8 + Math.random()*3,
    p10: 45 + Math.sin(i/6)*6 - 5,
    p90: 80 + Math.sin(i/6)*6 + 5,
    avg: 62
  }));
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="m" tickFormatter={(m)=>`M+${m}`} />
          <YAxis tickFormatter={(v)=>`€${v}`}/>
          <Tooltip formatter={(v,n)=>[typeof v==='number'?formatPrice(v):v,n]} />
          <Legend verticalAlign="bottom" height={24} />
          <Area type="monotone" dataKey={(d)=>d.p90} name="P90" stroke="#cbd5e1" fill="#cbd5e166" />
          <Area type="monotone" dataKey={(d)=>d.p10} name="P10" stroke="#cbd5e1" fill="#ffffff" />
          <Line type="monotone" dataKey="avg" name="Historical avg" stroke="#475569" strokeDasharray="4 4" dot={false}/>
          <Line type="monotone" dataKey="current" name="Current forward" stroke="#0ea5e9" strokeWidth={2} dot={false}/>
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

function TradesTable(){
  const rows = Array.from({length: 32}, (_,i)=>({
    ts: Date.now()-i*90*1000,
    product: `PL Hour ${String(12+(i%6)).padStart(2,'0')}`,
    price: 45 + Math.random()*30,
    volume: Math.round(1+Math.random()*10)*0.1,
    venue: ['EPEX','TGE','OTC'][i%3]
  }));
  return (
    <div className="overflow-auto">
      <table className="min-w-full text-sm">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800">
            {['Timestamp','Product','Price (€/MWh)','Volume (MW)','Venue'].map(h=> <th key={h} className="text-left px-3 py-2 font-medium">{h}</th>)}
          </tr>
        </thead>
        <tbody>
          {rows.map((r,i)=> (
            <tr key={i} className="odd:bg-white even:bg-slate-50 dark:odd:bg-slate-900 dark:even:bg-slate-800 hover:bg-slate-100/70 dark:hover:bg-slate-700/50">
              <td className="px-3 py-2 whitespace-nowrap">{tsFmt(r.ts)}</td>
              <td className="px-3 py-2">{r.product}</td>
              <td className="px-3 py-2 font-mono tabular-nums">{formatPrice(r.price)}</td>
              <td className="px-3 py-2 font-mono tabular-nums">{r.volume.toFixed(1)}</td>
              <td className="px-3 py-2">{r.venue}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="flex items-center justify-between mt-3 text-xs">
        <div className="opacity-70">Rows: 32</div>
        <div className="flex items-center gap-2">
          <button className="px-2 py-1 rounded border dark:border-slate-700">Prev</button>
          <button className="px-2 py-1 rounded border dark:border-slate-700">Next</button>
          <button className="px-2 py-1 rounded border dark:border-slate-700"><Download size={14} className="inline mr-1"/> Export</button>
        </div>
      </div>
    </div>
  );
}

// ---------- SHORT-TERM PAGE COMPONENTS ----------
function LiveImbalance(){
  const data = useMockTimeseries(48, Date.now()-47*30*60*1000, 30*60*1000, 0, 100);
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="ts" tickFormatter={tsFmt} interval={7} />
          <YAxis />
          <Tooltip labelFormatter={(l)=>tsFmt(l)} />
          <Legend verticalAlign="bottom" height={24} />
          <Area type="monotone" dataKey="value" stroke="#0ea5e9" fill="#0ea5e933" name="Imbalance" />
          <ReferenceLine y={0} stroke="#64748b" strokeDasharray="4 4" />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

function SpreadChart(){
  const data = useMockTimeseries(24, Date.now()-23*3600*1000, 3600*1000, 15, 8);
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="ts" tickFormatter={tsFmt} interval={3} />
          <YAxis />
          <Tooltip labelFormatter={(l)=>tsFmt(l)} />
          <Bar dataKey="value" fill="#0891b2" name="Spread" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}

function DispatchHelper(){
  const data = useMockTimeseries(24, Date.now(), 3600*1000, 0, 100);
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="ts" tickFormatter={tsFmt} interval={3} />
          <YAxis />
          <Tooltip labelFormatter={(l)=>tsFmt(l)} />
          <Legend verticalAlign="bottom" height={24} />
          <Line type="monotone" dataKey="value" stroke="#10b981" name="Arbitrage signal" strokeWidth={2} />
          <Area type="monotone" dataKey="value" stroke="#10b981" fill="#10b98122" name="SoC constraint" />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

// ---------- MID-TERM PAGE COMPONENTS ----------
function WeeklySeasonality(){
  const data = Array.from({length: 7}, (_,d)=> Array.from({length: 24}, (_,h)=> ({
    day: d,
    hour: h,
    price: 50 + Math.sin(h/12*Math.PI)*15 + Math.cos(d/7*Math.PI)*5 + Math.random()*3
  }))).flat();
  
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="hour" tickFormatter={(h)=>`${String(h).padStart(2,'0')}:00`} />
          <YAxis />
          <Tooltip formatter={(v)=>[formatPrice(v), 'Price']} />
          <Legend verticalAlign="bottom" height={24} />
          <Line type="monotone" dataKey="price" stroke="#0ea5e9" name="Median price" />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

function ReserveTrends(){
  const data = useMockTimeseries(84, Date.now()-83*24*3600*1000, 24*3600*1000, 150, 50);
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="ts" tickFormatter={tsFmt} interval={13} />
          <YAxis />
          <Tooltip labelFormatter={(l)=>tsFmt(l)} />
          <Legend verticalAlign="bottom" height={24} />
          <Line type="monotone" dataKey="value" stroke="#f59e0b" name="FCR" />
          <Line type="monotone" dataKey={(d)=>d.value*1.2} stroke="#ef4444" name="aFRR" />
          <Line type="monotone" dataKey={(d)=>d.value*0.8} stroke="#10b981" name="mFRR" />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

// ---------- LONG-TERM PAGE COMPONENTS ----------
function ScenarioPlanner(){
  const data = Array.from({length: 60}, (_,i)=>({
    month: i,
    base: 60 + Math.sin(i/12*Math.PI)*10 + i*0.5,
    highRes: 55 + Math.sin(i/12*Math.PI)*8 + i*0.3,
    gasShock: 80 + Math.sin(i/12*Math.PI)*15 + i*1.2
  }));
  
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="month" tickFormatter={(m)=>`M+${m}`} />
          <YAxis />
          <Tooltip formatter={(v)=>[formatPrice(v), 'Price']} />
          <Legend verticalAlign="bottom" height={24} />
          <Line type="monotone" dataKey="base" stroke="#0ea5e9" name="Base case" strokeWidth={2} />
          <Line type="monotone" dataKey="highRes" stroke="#10b981" name="High RES" strokeWidth={2} />
          <Line type="monotone" dataKey="gasShock" stroke="#ef4444" name="Gas shock" strokeWidth={2} />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

function Fundamentals(){
  const data = useMockTimeseries(60, Date.now()-59*24*3600*1000, 24*3600*1000, 100, 30);
  return (
    <div className="h-72">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid stroke="#e5e7eb" />
          <XAxis dataKey="ts" tickFormatter={tsFmt} interval={14} />
          <YAxis />
          <Tooltip labelFormatter={(l)=>tsFmt(l)} />
          <Legend verticalAlign="bottom" height={24} />
          <Line type="monotone" dataKey="value" stroke="#ea580c" name="Natural gas" />
          <Line type="monotone" dataKey={(d)=>d.value*0.7} stroke="#6b7280" name="Coal" />
          <Line type="monotone" dataKey={(d)=>d.value*0.3} stroke="#059669" name="CO₂" />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}

// ---------- ALERTS PAGE COMPONENTS ----------
function AlertsTable({ onCreate }){
  const alerts = [
    { id: 1, name: 'Price spike > 100€/MWh', type: 'price', threshold: 100, active: true, lastTriggered: Date.now()-3600000 },
    { id: 2, name: 'Reserve shortage', type: 'reserve', threshold: 50, active: true, lastTriggered: Date.now()-7200000 },
    { id: 3, name: 'Cross-border flow reversal', type: 'flow', threshold: 0, active: false, lastTriggered: null },
  ];
  
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="font-medium">Active Rules</h4>
        <button onClick={onCreate} className="px-3 py-1.5 rounded text-white text-sm" style={{ backgroundColor: COLORS.secondary }}>Create Alert</button>
      </div>
      <div className="overflow-auto">
        <table className="min-w-full text-sm">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800">
              <th className="text-left px-3 py-2 font-medium">Name</th>
              <th className="text-left px-3 py-2 font-medium">Type</th>
              <th className="text-left px-3 py-2 font-medium">Threshold</th>
              <th className="text-left px-3 py-2 font-medium">Status</th>
              <th className="text-left px-3 py-2 font-medium">Last Triggered</th>
              <th className="text-left px-3 py-2 font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            {alerts.map((alert,i)=> (
              <tr key={alert.id} className="odd:bg-white even:bg-slate-50 dark:odd:bg-slate-900 dark:even:bg-slate-800">
                <td className="px-3 py-2">{alert.name}</td>
                <td className="px-3 py-2">{alert.type}</td>
                <td className="px-3 py-2 font-mono tabular-nums">{alert.threshold}</td>
                <td className="px-3 py-2">
                  <span className={classNames("px-2 py-0.5 rounded-full text-xs", alert.active ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300" : "bg-gray-100 text-gray-700 dark:bg-gray-900/30 dark:text-gray-300")}>
                    {alert.active ? "Active" : "Inactive"}
                  </span>
                </td>
                <td className="px-3 py-2 text-xs">
                  {alert.lastTriggered ? tsFmt(alert.lastTriggered) : "Never"}
                </td>
                <td className="px-3 py-2">
                  <button className="text-xs underline">Edit</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function AlertModal({ onClose }){
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-slate-900 rounded-lg shadow-xl max-w-md w-full mx-4">
        <div className="p-6">
          <h3 className="text-lg font-semibold mb-4">Create New Alert</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-1">Alert Name</label>
              <input type="text" className="w-full px-3 py-2 rounded border dark:border-slate-700 bg-white dark:bg-slate-800" placeholder="e.g., Price spike detection" />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Metric</label>
              <select className="w-full px-3 py-2 rounded border dark:border-slate-700 bg-white dark:bg-slate-800">
                <option>Spot Price</option>
                <option>Reserve Price</option>
                <option>Cross-border Flow</option>
                <option>Grid Balance</option>
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Threshold</label>
              <input type="number" className="w-full px-3 py-2 rounded border dark:border-slate-700 bg-white dark:bg-slate-800" placeholder="100" />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Condition</label>
              <select className="w-full px-3 py-2 rounded border dark:border-slate-700 bg-white dark:bg-slate-800">
                <option>Greater than</option>
                <option>Less than</option>
                <option>Equals</option>
              </select>
            </div>
          </div>
          <div className="flex items-center gap-3 mt-6">
            <button onClick={onClose} className="flex-1 px-4 py-2 rounded border dark:border-slate-700">Cancel</button>
            <button className="flex-1 px-4 py-2 rounded text-white" style={{ backgroundColor: COLORS.secondary }}>Create Alert</button>
          </div>
        </div>
      </div>
    </div>
  );
}

// ---------- FLOATING ACTIONS ----------
function FloatingActions(){
  const [expanded, setExpanded] = useState(false);
  
  return (
    <div className="fixed bottom-6 right-6 z-40">
      <div className={classNames("flex flex-col gap-3 transition-all", expanded ? "items-end" : "items-center")}>
        {expanded && (
          <>
            <button className="px-4 py-2 rounded-full bg-white dark:bg-slate-800 shadow-lg border dark:border-slate-700 text-sm whitespace-nowrap">
              <Settings size={16} className="inline mr-2"/> Settings
            </button>
            <button className="px-4 py-2 rounded-full bg-white dark:bg-slate-800 shadow-lg border dark:border-slate-700 text-sm whitespace-nowrap">
              <Share2 size={16} className="inline mr-2"/> Share View
            </button>
            <button className="px-4 py-2 rounded-full bg-white dark:bg-slate-800 shadow-lg border dark:border-slate-700 text-sm whitespace-nowrap">
              <Download size={16} className="inline mr-2"/> Export Data
            </button>
          </>
        )}
        <button 
          onClick={() => setExpanded(!expanded)}
          className="w-14 h-14 rounded-full text-white shadow-lg flex items-center justify-center hover:opacity-90"
          style={{ backgroundColor: COLORS.secondary }}
        >
          {expanded ? <X size={24}/> : <AlertTriangle size={24}/>}
        </button>
      </div>
    </div>
  );
}

// ---------- FOOTER ----------
function Footer(){
  return (
    <footer className="mt-12 border-t dark:border-slate-800 bg-white dark:bg-slate-900">
      <div className="max-w-7xl mx-auto px-4 md:px-6 lg:px-8 py-6">
        <div className="flex flex-col md:flex-row items-center justify-between gap-4">
          <div className="flex items-center gap-6 text-sm opacity-70">
            <span>© 2024 GridLens. All rights reserved.</span>
            <div className="flex items-center gap-4">
              <a href="#" className="hover:opacity-100">API Docs</a>
              <a href="#" className="hover:opacity-100">Terms</a>
              <a href="#" className="hover:opacity-100">Privacy</a>
              <a href="#" className="hover:opacity-100">Support</a>
            </div>
          </div>
          <div className="flex items-center gap-4 text-sm">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-green-500"></div>
              <span>System Status: Operational</span>
            </div>
            <span className="opacity-70">v2.1.4</span>
          </div>
        </div>
      </div>
    </footer>
  );
}

// ---------- CSS ANIMATIONS ----------
const styles = `
  @keyframes slide-in {
    from { transform: translateY(-100%); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
  .animate-slide-in {
    animation: slide-in 0.3s ease-out;
  }
`;

// Inject styles
if (typeof document !== 'undefined') {
  const styleSheet = document.createElement('style');
  styleSheet.textContent = styles;
  document.head.appendChild(styleSheet);
}

