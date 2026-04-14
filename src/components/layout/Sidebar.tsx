import { NavLink } from "react-router-dom";
import { FolderKanban, LayoutDashboard, DollarSign, Activity, Settings } from "lucide-react";

const navItems = [
  { to: "/", icon: FolderKanban, label: "Projects" },
  { to: "/setup", icon: Settings, label: "Setup" },
  { to: "/monitoring", icon: Activity, label: "Monitoring", disabled: true },
  { to: "/costs", icon: DollarSign, label: "Cost Explorer", disabled: true },
];

export function Sidebar() {
  return (
    <aside className="flex h-screen w-60 flex-col border-r border-gray-200 bg-white">
      <div className="flex h-16 items-center gap-2.5 border-b border-gray-200 px-5">
        <LayoutDashboard className="h-6 w-6 text-brand-600" />
        <span className="text-lg font-semibold tracking-tight text-gray-900">
          Operra
        </span>
      </div>

      <nav className="flex-1 space-y-1 px-3 py-4">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.disabled ? "#" : item.to}
            end={item.to === "/"}
            onClick={(e) => item.disabled && e.preventDefault()}
            className={({ isActive }) =>
              `flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
                item.disabled
                  ? "cursor-not-allowed text-gray-400"
                  : isActive
                    ? "bg-brand-50 text-brand-700"
                    : "text-gray-600 hover:bg-gray-100 hover:text-gray-900"
              }`
            }
          >
            <item.icon className="h-4.5 w-4.5" />
            {item.label}
            {item.disabled && (
              <span className="ml-auto text-[10px] font-normal text-gray-400">
                Soon
              </span>
            )}
          </NavLink>
        ))}
      </nav>

      <div className="border-t border-gray-200 px-5 py-3">
        <p className="text-xs text-gray-400">Operra v0.1.0</p>
      </div>
    </aside>
  );
}
