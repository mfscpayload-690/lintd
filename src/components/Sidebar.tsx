import { History, LayoutDashboard, Moon, Package, Sun, Trash2 } from "lucide-react";
import { NavLink } from "react-router-dom";
import { useThemeStore } from "../lib/theme-store";

const APP_VERSION = "v1.0.0";

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
    <aside className="fixed left-0 top-0 h-screen w-[200px] border-r bg-card px-4 py-5">
      <div className="mb-8 flex items-center gap-2">
        <img
          src="/app-icon.png"
          alt="Lintd logo"
          className="h-9 w-9 rounded-md border object-cover"
        />
        <div className="text-lg font-semibold leading-none">Lintd</div>
      </div>

      <nav className="space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          return (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                isActive
                  ? "flex items-center gap-2 border-l-2 border-primary pl-2 py-2 text-sm text-primary"
                  : "flex items-center gap-2 pl-3 py-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
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
        <button
          type="button"
          className="text-sm text-muted-foreground hover:text-foreground"
          onClick={toggle}
        >
          {mode === "dark" ? <Sun className="h-4 w-4 inline mr-1" /> : <Moon className="h-4 w-4 inline mr-1" />}
          {mode === "dark" ? "Light" : "Dark"}
        </button>
        <div className="text-xs text-muted-foreground">Lintd {APP_VERSION}</div>
      </div>
    </aside>
  );
}
