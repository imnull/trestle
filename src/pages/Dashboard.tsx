import { useStore } from "../stores/app";
import { Activity, Server, Route, Clock } from "lucide-react";

export function Dashboard() {
  const { serverStatus, providers, routes, connected } = useStore();

  const stats = [
    {
      label: "服务商",
      value: providers.length,
      icon: Server,
      color: "text-blue-500",
      bg: "bg-blue-50",
    },
    {
      label: "路由规则",
      value: routes.length,
      icon: Route,
      color: "text-purple-500",
      bg: "bg-purple-50",
    },
    {
      label: "活跃连接",
      value: serverStatus?.active_connections || 0,
      icon: Activity,
      color: "text-green-500",
      bg: "bg-green-50",
    },
    {
      label: "运行时间",
      value: formatUptime(serverStatus?.uptime_secs || 0),
      icon: Clock,
      color: "text-orange-500",
      bg: "bg-orange-50",
    },
  ];

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">仪表盘</h1>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <div
              key={stat.label}
              className="bg-white rounded-xl border border-gray-200 p-5"
            >
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">{stat.label}</p>
                  <p className="text-2xl font-bold mt-1">{stat.value}</p>
                </div>
                <div className={`p-3 rounded-lg ${stat.bg}`}>
                  <Icon className={`w-6 h-6 ${stat.color}`} />
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Server Status */}
      <div className="bg-white rounded-xl border border-gray-200 p-5">
        <h2 className="text-lg font-semibold mb-4">服务状态</h2>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="text-sm text-gray-500">状态</p>
            <p className={`font-medium ${connected ? "text-green-600" : "text-red-500"}`}>
              {connected ? "✓ 正常运行" : "✗ 未连接"}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500">端口</p>
            <p className="font-medium">{serverStatus?.port || 31415}</p>
          </div>
          <div>
            <p className="text-sm text-gray-500">代理地址</p>
            <p className="font-medium font-mono text-sm">
              http://127.0.0.1:{serverStatus?.port || 31415}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500">API 地址</p>
            <p className="font-medium font-mono text-sm">
              http://127.0.0.1:{serverStatus?.port || 31415}/api
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function formatUptime(secs: number): string {
  if (secs < 60) return `${secs}秒`;
  if (secs < 3600) return `${Math.floor(secs / 60)}分钟`;
  if (secs < 86400) return `${Math.floor(secs / 3600)}小时`;
  return `${Math.floor(secs / 86400)}天`;
}
