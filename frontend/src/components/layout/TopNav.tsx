export default function TopNav() {
  return (
    <header className="fixed top-0 w-full z-50 bg-background flex justify-between items-center px-6 h-16 font-headline text-sm tracking-tight">
      <div className="flex items-center gap-8">
        <span className="text-xl font-bold text-primary tracking-tight">
          Plentra Research
        </span>
        <nav className="hidden md:flex gap-6 items-center">
          <a
            className="text-slate-400 hover:bg-surface-container-high transition-colors px-3 py-1 rounded"
            href="#"
          >
            Markets
          </a>
          <a
            className="text-slate-400 hover:bg-surface-container-high transition-colors px-3 py-1 rounded"
            href="#"
          >
            Intelligence
          </a>
        </nav>
      </div>
      <div className="flex items-center gap-4">
        <div className="relative flex items-center bg-surface-container-low rounded-lg px-3 py-1.5 w-64">
          <span className="material-symbols-outlined text-outline text-sm mr-2">
            search
          </span>
          <input
            className="bg-transparent border-none focus:ring-0 focus:outline-none text-xs w-full text-on-surface placeholder:text-outline"
            placeholder="Search markets..."
            type="text"
          />
        </div>
        <span className="material-symbols-outlined text-slate-400 cursor-pointer hover:text-primary transition-colors">
          notifications
        </span>
        <span className="material-symbols-outlined text-slate-400 cursor-pointer hover:text-primary transition-colors">
          calendar_month
        </span>
        <div className="w-8 h-8 rounded-full bg-primary-container flex items-center justify-center text-on-primary-container text-xs font-bold">
          PL
        </div>
      </div>
    </header>
  );
}
