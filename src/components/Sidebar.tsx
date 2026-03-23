import { useStore } from "../stores/app";
import {
  LayoutDashboard,
  Server,
  Route,
  FileText,
  Settings,
  Zap,
} from "lucide-react";
import { cn } from "../lib/utils";

const navItems = [
  { id: "dashboard" as const, label: "仪表盘", icon: LayoutDashboard },
  { id: "providers" as const, label: "服务商", icon: Server },
  { id: "routes" as const, label: "路由", icon: Route },
  { id: "logs" as const, label: "日志", icon: FileText },
  { id: "settings" as const, label: "设置", icon: Settings },
];

export function Sidebar() {
  const { currentPage, setCurrentPage, serverStatus, connected } = useStore();

  return (
    <aside className="w-56 bg-white border-r border-gray-200 flex flex-col">
      {/* Logo */}
      <div className="h-16 flex items-center px-5 border-b border-gray-200">
        <Zap className="w-6 h-6 text-primary-500 mr-2" />
        <span className="text-lg font-semibold">Trestle</span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-3">
        <ul className="space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = currentPage === item.id;
            return (
              <li key={item.id}>
                <button
                  onClick={() => setCurrentPage(item.id)}
                  className={cn(
                    "w-full flex items-center px-3 py-2.5 rounded-lg text-sm font-medium transition-colors",
                    isActive
                      ? "bg-primary-50 text-primary-600"
                      : "text-gray-600 hover:bg-gray-50 hover:text-gray-900"
                  )}
                >
                  <Icon className="w-5 h-5 mr-3" />
                  {item.label}
                </button>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Status */}
      <div className="p-4 border-t border-gray-200">
        <div className="flex items-center text-sm">
          <span
            className={cn(
              "w-2 h-2 rounded-full mr-2",
              connected ? "bg-green-500" : "bg-gray-300"
            )}
          />
          <span className={connected ? "text-green-600" : "text-gray-500"}>
            {connected ? "运行中" : "未连接"}
          </span>
        </div>
        {serverStatus && (
          <p className="text-xs text-gray-400 mt-1">
            端口: {serverStatus.port}
          </p>
        )}
      </div>
    </aside>
  );
}
