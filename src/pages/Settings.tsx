import { useState } from "react";

export function Settings() {
  const [settings, setSettings] = useState({
    port: 31415,
    logLevel: "info",
    autoStart: true,
    minimizeToTray: true,
    theme: "system",
  });

  const handleSave = () => {
    // TODO: 保存设置到 Tauri store
    alert("设置已保存");
  };

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">设置</h1>

      <div className="bg-white rounded-xl border border-gray-200 p-6 space-y-6">
        {/* 服务器设置 */}
        <section>
          <h2 className="text-lg font-semibold mb-4">服务器</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                端口
              </label>
              <input
                type="number"
                value={settings.port}
                onChange={(e) =>
                  setSettings({ ...settings, port: parseInt(e.target.value) })
                }
                className="w-32 px-3 py-2 border border-gray-300 rounded-lg"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                日志级别
              </label>
              <select
                value={settings.logLevel}
                onChange={(e) =>
                  setSettings({ ...settings, logLevel: e.target.value })
                }
                className="w-48 px-3 py-2 border border-gray-300 rounded-lg"
              >
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warn</option>
                <option value="error">Error</option>
              </select>
            </div>
          </div>
        </section>

        {/* 应用设置 */}
        <section className="pt-6 border-t border-gray-200">
          <h2 className="text-lg font-semibold mb-4">应用</h2>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">开机自启动</p>
                <p className="text-sm text-gray-500">
                  系统启动时自动运行 Trestle
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.autoStart}
                  onChange={(e) =>
                    setSettings({ ...settings, autoStart: e.target.checked })
                  }
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
              </label>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">最小化到托盘</p>
                <p className="text-sm text-gray-500">
                  关闭窗口时最小化到系统托盘
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.minimizeToTray}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      minimizeToTray: e.target.checked,
                    })
                  }
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
              </label>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                主题
              </label>
              <select
                value={settings.theme}
                onChange={(e) =>
                  setSettings({ ...settings, theme: e.target.value })
                }
                className="w-48 px-3 py-2 border border-gray-300 rounded-lg"
              >
                <option value="system">跟随系统</option>
                <option value="light">浅色</option>
                <option value="dark">深色</option>
              </select>
            </div>
          </div>
        </section>

        {/* 保存按钮 */}
        <div className="pt-6 border-t border-gray-200">
          <button
            onClick={handleSave}
            className="px-6 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600"
          >
            保存设置
          </button>
        </div>
      </div>
    </div>
  );
}
