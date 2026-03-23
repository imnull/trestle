import { useState } from "react";
import { useStore } from "../stores/app";
import { Plus, Pencil, Trash2, Power } from "lucide-react";
import type { Route } from "../stores/app";

export function Routes() {
  const { routes, providers, addRoute, updateRoute, deleteRoute } = useStore();
  const [editing, setEditing] = useState<Route | null>(null);
  const [showForm, setShowForm] = useState(false);

  const emptyRoute: Route = {
    id: "",
    path: "",
    target_provider: "",
    target_model: "",
    enabled: true,
  };

  const handleSave = async (route: Route) => {
    if (editing) {
      await updateRoute(editing.id, route);
    } else {
      await addRoute({ ...route, id: crypto.randomUUID() });
    }
    setShowForm(false);
    setEditing(null);
  };

  const handleEdit = (route: Route) => {
    setEditing(route);
    setShowForm(true);
  };

  const handleDelete = async (id: string) => {
    if (confirm("确定要删除这条路由吗？")) {
      await deleteRoute(id);
    }
  };

  const handleToggle = async (route: Route) => {
    await updateRoute(route.id, { ...route, enabled: !route.enabled });
  };

  const getProviderName = (id: string) => {
    return providers.find((p) => p.id === id)?.name || id;
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900">路由规则</h1>
        <button
          onClick={() => {
            setEditing(null);
            setShowForm(true);
          }}
          className="flex items-center px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
        >
          <Plus className="w-4 h-4 mr-2" />
          添加路由
        </button>
      </div>

      {/* Route List */}
      <div className="space-y-3">
        {routes.map((route) => (
          <div
            key={route.id}
            className="bg-white rounded-xl border border-gray-200 p-4 flex items-center justify-between"
          >
            <div className="flex items-center gap-4">
              <button
                onClick={() => handleToggle(route)}
                className={`p-2 rounded-lg transition-colors ${
                  route.enabled
                    ? "bg-green-100 text-green-600"
                    : "bg-gray-100 text-gray-400"
                }`}
              >
                <Power className="w-5 h-5" />
              </button>
              <div>
                <h3 className="font-mono text-sm bg-gray-100 px-2 py-1 rounded">
                  {route.path}
                </h3>
                <p className="text-sm text-gray-500 mt-1">
                  → {getProviderName(route.target_provider)} / {route.target_model}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => handleEdit(route)}
                className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
              >
                <Pencil className="w-4 h-4" />
              </button>
              <button
                onClick={() => handleDelete(route.id)}
                className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          </div>
        ))}

        {routes.length === 0 && (
          <div className="text-center py-12 text-gray-400">
            暂无路由规则，点击上方按钮添加
          </div>
        )}
      </div>

      {/* Form Modal */}
      {showForm && (
        <RouteForm
          route={editing || emptyRoute}
          isNew={!editing}
          providers={providers}
          onSave={handleSave}
          onClose={() => {
            setShowForm(false);
            setEditing(null);
          }}
        />
      )}
    </div>
  );
}

function RouteForm({
  route,
  isNew,
  providers,
  onSave,
  onClose,
}: {
  route: Route;
  isNew: boolean;
  providers: { id: string; name: string; models: string[] }[];
  onSave: (r: Route) => void;
  onClose: () => void;
}) {
  const [form, setForm] = useState(route);
  const selectedProvider = providers.find((p) => p.id === form.target_provider);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(form);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl w-full max-w-lg p-6">
        <h2 className="text-xl font-bold mb-4">
          {isNew ? "添加路由" : "编辑路由"}
        </h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              路径
            </label>
            <input
              type="text"
              value={form.path}
              onChange={(e) => setForm({ ...form, path: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent font-mono"
              placeholder="/v1/chat/completions"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              目标服务商
            </label>
            <select
              value={form.target_provider}
              onChange={(e) => setForm({ ...form, target_provider: e.target.value, target_model: "" })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              required
            >
              <option value="">选择服务商</option>
              {providers.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              目标模型
            </label>
            <select
              value={form.target_model}
              onChange={(e) => setForm({ ...form, target_model: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              required
              disabled={!selectedProvider}
            >
              <option value="">选择模型</option>
              {selectedProvider?.models.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          </div>
          <div className="flex items-center">
            <input
              type="checkbox"
              id="route-enabled"
              checked={form.enabled}
              onChange={(e) => setForm({ ...form, enabled: e.target.checked })}
              className="w-4 h-4 text-primary-500 border-gray-300 rounded focus:ring-primary-500"
            />
            <label htmlFor="route-enabled" className="ml-2 text-sm text-gray-700">
              启用此路由
            </label>
          </div>
          <div className="flex justify-end gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg"
            >
              取消
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600"
            >
              保存
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
