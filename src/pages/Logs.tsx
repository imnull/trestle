import { useEffect } from "react";
import { useStore } from "../stores/app";

export function Logs() {
  const { logs, fetchLogs } = useStore();

  useEffect(() => {
    fetchLogs();
    // 定期刷新
    const interval = setInterval(fetchLogs, 3000);
    return () => clearInterval(interval);
  }, [fetchLogs]);

  const getLevelColor = (level: string) => {
    switch (level.toLowerCase()) {
      case "error":
        return "text-red-600 bg-red-50";
      case "warn":
        return "text-yellow-600 bg-yellow-50";
      case "info":
        return "text-blue-600 bg-blue-50";
      case "debug":
        return "text-gray-500 bg-gray-50";
      default:
        return "text-gray-600 bg-gray-50";
    }
  };

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">日志</h1>

      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        <div className="max-h-[calc(100vh-200px)] overflow-auto">
          <table className="w-full text-sm">
            <thead className="bg-gray-50 sticky top-0">
              <tr>
                <th className="text-left px-4 py-3 font-medium text-gray-500 w-44">
                  时间
                </th>
                <th className="text-left px-4 py-3 font-medium text-gray-500 w-20">
                  级别
                </th>
                <th className="text-left px-4 py-3 font-medium text-gray-500 w-32">
                  请求 ID
                </th>
                <th className="text-left px-4 py-3 font-medium text-gray-500">
                  消息
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {logs.map((log, i) => (
                <tr key={i} className="hover:bg-gray-50">
                  <td className="px-4 py-2 text-gray-500 font-mono text-xs">
                    {log.timestamp}
                  </td>
                  <td className="px-4 py-2">
                    <span
                      className={`px-2 py-0.5 rounded text-xs font-medium ${getLevelColor(
                        log.level
                      )}`}
                    >
                      {log.level}
                    </span>
                  </td>
                  <td className="px-4 py-2 text-gray-400 font-mono text-xs">
                    {log.request_id || "-"}
                  </td>
                  <td className="px-4 py-2 text-gray-700">{log.message}</td>
                </tr>
              ))}
            </tbody>
          </table>

          {logs.length === 0 && (
            <div className="text-center py-12 text-gray-400">暂无日志</div>
          )}
        </div>
      </div>
    </div>
  );
}
