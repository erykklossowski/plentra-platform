import { cn } from "@/lib/utils";
import type { ReactNode } from "react";

interface SectionModuleProps {
  title: string;
  subtitle?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
}

export default function SectionModule({
  title,
  subtitle,
  action,
  children,
  className,
}: SectionModuleProps) {
  return (
    <div className={cn("bg-surface-container p-6 rounded-xl", className)}>
      <div className="flex items-start justify-between">
        <div>
          <h2 className="font-headline text-lg font-bold text-on-surface">
            {title}
          </h2>
          {subtitle && (
            <p className="text-sm text-on-surface-variant mt-0.5">
              {subtitle}
            </p>
          )}
        </div>
        {action && <div>{action}</div>}
      </div>
      <div className="mt-4">{children}</div>
    </div>
  );
}
