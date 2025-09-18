import * as React from "react";
import { ChevronRight, Home } from "lucide-react";
import { cn } from "@/utils/cn";
import Link from "next/link";

interface BreadcrumbItem {
  label: string;
  href?: string;
  current?: boolean;
}

interface BreadcrumbsProps {
  items: BreadcrumbItem[];
  separator?: React.ReactNode;
  className?: string;
}

export const Breadcrumbs = React.forwardRef<HTMLElement, BreadcrumbsProps>(
  ({ items, separator, className, ...props }, ref) => {
    return (
      <nav
        ref={ref}
        aria-label="Breadcrumb"
        className={cn("flex", className)}
        {...props}
      >
        <ol className="flex items-center space-x-1 md:space-x-3">
          {items.map((item, index) => (
            <li key={index} className="flex items-center">
              {index > 0 && (
                <div className="flex items-center">
                  {separator || (
                    <ChevronRight className="h-4 w-4 text-muted-foreground mx-1" />
                  )}
                </div>
              )}
              <div className="flex items-center">
                {index === 0 && item.href && (
                  <Home className="h-4 w-4 text-muted-foreground mr-1" />
                )}
                {item.href && !item.current ? (
                  <a
                    href={item.href}
                    className="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                  >
                    {item.label}
                  </a>
                ) : (
                  <span
                    className={cn(
                      "text-sm font-medium",
                      item.current
                        ? "text-foreground"
                        : "text-muted-foreground"
                    )}
                    aria-current={item.current ? "page" : undefined}
                  >
                    {item.label}
                  </span>
                )}
              </div>
            </li>
          ))}
        </ol>
      </nav>
    );
  }
);

Breadcrumbs.displayName = "Breadcrumbs";
