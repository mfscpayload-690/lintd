import { History, LayoutDashboard, Moon, Package, Sun, Trash2 } from "lucide-react";
import { NavLink } from "react-router-dom";
import { Button } from "./ui/button";
import { useThemeStore } from "../lib/theme-store";
import { cn } from "../lib/utils";

const APP_VERSION = "v0.1.0";

const navItems = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/packages", label: "Packages", icon: Package },
  { to: "/orphans", label: "Orphans", icon: Trash2 },
  { to: "/history", label: "History", icon: History },
] as const;

export function Sidebar() {
  const mode = useThemeStore((state) => state.mode);
  const toggle = useThemeStore((state) => state.toggle);

  return (
    <aside className="fixed left-0 top-0 h-screen w-[220px] border-r bg-card px-4 py-5">
      <div className="mb-8 flex items-center gap-2">
        <div className="inline-flex h-9 w-9 items-center justify-center rounded-md bg-primary text-primary-foreground font-semibold">
          L
        </div>
        <div>
          <div className="text-lg font-semibold leading-none">Lintd</div>
          <div className="text-xs text-muted-foreground">Package Auditor</div>
        </div>
      </div>

      <nav className="space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          return (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )
              }
              end={item.to === "/"}
            >
              <Icon className="h-4 w-4" />
              <span>{item.label}</span>
            </NavLink>
          );
        })}
      </nav>

      <div className="absolute bottom-5 left-4 right-4 space-y-3">
        <Button variant="outline" className="w-full justify-start" onClick={toggle}>
          {mode === "dark" ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
          <span>{mode === "dark" ? "Light mode" : "Dark mode"}</span>
        </Button>
        <div className="text-xs text-muted-foreground">Lintd {APP_VERSION}</div>
      </div>
    </aside>
  );
}
