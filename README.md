# GridLens - Energy Price Observability Platform

A sophisticated, modern web application for energy price observability and predictability designed for Commercial & Industrial (C&I) energy consumers. This platform provides comprehensive visibility into electricity market dynamics across multiple time horizons, from real-time grid operations to 5-year macroeconomic forecasts.

## 🚀 Features

### Core Functionality
- **Real-Time Dashboard**: Live spot prices, grid balance, and market dynamics
- **Multi-Timeframe Analysis**: Short-term (0-48h), Mid-term (weekly), and Long-term (5-year) views
- **Advanced Visualizations**: Interactive charts, heatmaps, and geographic flows
- **Alert System**: Configurable price and market condition alerts
- **Data Export**: CSV/Excel export capabilities
- **Dark Mode**: Full dark mode support with optimized contrast

### Key Components

#### Dashboard Metrics
- Current Spot Price with trend indicators
- Day-Ahead Average with comparisons
- Weekly Forecast with min/max ranges
- Monthly Outlook with volatility indices

#### Real-Time Visualizations
- **Grid Balance Chart**: Stacked area showing generation mix vs demand
- **Intraday Price Curve**: OHLC candlestick charts with volume and VWAP
- **Reserve Market Gauges**: FCR, aFRR, mFRR prices with historical sparklines
- **Cross-Border Flows**: Animated European map with flow arrows
- **Merit Order Curve**: Generation stack with marginal costs

#### Advanced Analytics
- **Price Heatmap**: 7-day hourly price visualization
- **Weather Impact**: Wind, solar, temperature, and precipitation forecasts
- **Forward Curve**: Monthly forwards with historical envelopes
- **Scenario Planning**: Base, High RES, and Gas Shock scenarios

## 🎨 Design System

### Color Palette
- **Primary**: Deep Blue (`#1e3a8a`) - Headers and primary actions
- **Secondary**: Teal (`#0891b2`) - Interactive elements
- **Accent**: Amber (`#f59e0b`) - Alerts and warnings
- **Success**: Green (`#10b981`) - Positive indicators
- **Danger**: Red (`#ef4444`) - Critical alerts

### Typography
- **Headers**: Inter font family
- **Data/Numbers**: Tabular nums, monospace for prices
- **Responsive**: 8px grid system with consistent spacing

### Layout
- **Top Navigation**: Fixed 64px height with spot price ticker
- **Left Sidebar**: Collapsible 240px width with filters
- **Main Content**: 12-column responsive grid system
- **Mobile-First**: Responsive design for all screen sizes

## 🛠️ Technology Stack

- **Frontend**: React 18 with Hooks
- **Styling**: Tailwind CSS with custom design system
- **Charts**: Recharts for data visualization
- **Icons**: Lucide React for consistent iconography
- **Build Tool**: Vite for fast development and optimized builds
- **Fonts**: Inter for modern, clean typography

## 📦 Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd plentra-platform
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Start development server**
   ```bash
   npm run dev
   ```

4. **Open in browser**
   Navigate to `http://localhost:3000`

## 🚀 Available Scripts

- `npm run dev` - Start development server with hot reload
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm start` - Alias for dev command

## 📱 Responsive Design

The platform is fully responsive with breakpoints:
- **Desktop**: 1440px+ (Full sidebar, 3-4 cards per row)
- **Tablet**: 768px-1439px (Collapsible sidebar, 2 cards per row)
- **Mobile**: 375px-767px (Hidden sidebar, single column layout)

## 🎯 Key Features for Energy Traders

### Real-Time Decision Making
- Live spot price monitoring with trend indicators
- Grid balance visualization for supply/demand insights
- Reserve market status for system stability awareness

### Risk Management
- Price volatility tracking and forecasting
- Cross-border flow monitoring for arbitrage opportunities
- Weather impact analysis for renewable generation

### Strategic Planning
- Forward curve analysis for hedging decisions
- Scenario planning for different market conditions
- Historical data analysis for pattern recognition

## 🔧 Technical Annotations

### API Endpoints (Mock)
- `/api/grid/balance` - Real-time grid balance data
- `/api/price/intraday_ohlc` - Intraday price data
- `/api/reserve/prices` - Reserve market prices
- `/api/flows/net` - Cross-border flow data
- `/api/merit_order` - Merit order curve data
- `/api/price/dayahead` - Day-ahead price forecasts
- `/api/weather/*` - Weather impact data
- `/api/forwards/monthly` - Forward curve data

### Data Update Frequencies
- **Real-time**: 15s WebSocket streams
- **Intraday**: 60s updates
- **Reserve markets**: 5m updates
- **Day-ahead**: Daily at 12:00
- **Forward curves**: End-of-day updates

### Performance Optimizations
- Lazy loading for chart components
- Memoized data calculations
- Optimized bundle splitting
- CDN caching for static assets

## 🎨 Customization

### Theme Configuration
The design system is fully customizable through the `tailwind.config.js` file:
- Color palette modifications
- Typography adjustments
- Spacing and layout changes
- Animation customizations

### Component Styling
All components use Tailwind CSS classes and can be easily modified:
- Card layouts and shadows
- Chart colors and styling
- Button and form elements
- Responsive breakpoints

## 📊 Data Sources (Mock)

This mockup includes realistic data patterns based on:
- European electricity market structures
- Typical price volatility patterns
- Weather-driven generation profiles
- Cross-border trading dynamics

## 🔮 Future Enhancements

- **Real API Integration**: Connect to actual market data sources
- **User Authentication**: Multi-user support with role-based access
- **Advanced Analytics**: Machine learning price predictions
- **Mobile App**: Native mobile application
- **API Documentation**: Swagger/OpenAPI documentation
- **WebSocket Integration**: Real-time data streaming
- **Export Formats**: PDF reports and advanced data exports

## 📄 License

MIT License - see LICENSE file for details

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📞 Support

For questions or support, please contact the development team or create an issue in the repository.

---

**GridLens** - Empowering energy traders with comprehensive market intelligence.
