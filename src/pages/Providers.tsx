import { useState } from "react";
import { useStore } from "../stores/app";
import { Plus, Pencil, Trash2, Power } from "lucide-react";
import type { Provider } from "../stores/app";

export function Providers() {
  const { providers, addProvider, updateProvider, deleteProvider } = useStore();
  const [editing, setEditing] = useState<Provider | null>(null);
  const [showForm, setShowForm] = useState(false);

  const emptyProvider: Provider = {
    id: "",
    name: "",
    api_base: "",
    api_key: "",
    models: [],
    enabled: true,
  };

  const handleSave = async (provider: Provider) => {
    if (editing) {
      await updateProvider(editing.id, provider);
    } else {
      await addProvider({ ...provider, id: crypto.randomUUID() });
    }
    setShowForm(false);
    setEditing(null);
  };

  const handleEdit = (provider: Provider) => {
    setEditing(provider);
    setShowForm(true);
  };

  const handleDelete = async (id: string) => {
    if (confirm("确定要删除这个服务商吗？")) {
      await deleteProvider(id);
    }
  };

  const handleToggle = async (provider: Provider) => {
    await updateProvider(provider.id, { ...provider, enabled: !provider.enabled });
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900">服务商</h1>
        <button
          onClick={() => {
            setEditing(null);
            setShowForm(true);
          }}
          className="flex items-center px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
        >
          <Plus className="w-4 h-4 mr-2" />
          添加服务商
        </button>
      </div>

      {/* Provider List */}
      <div className="space-y-3">
        {providers.map((provider) => (
          <div
            key={provider.id}
            className="bg-white rounded-xl border border-gray-200 p-4 flex items-center justify-between"
          >
            <div className="flex items-center gap-4">
              <button
                onClick={() => handleToggle(provider)}
                className={`p-2 rounded-lg transition-colors ${
                  provider.enabled
                    ? "bg-green-100 text-green-600"
                    : "bg-gray-100 text-gray-400"
                }`}
              >
                <Power className="w-5 h-5" />
              </button>
              <div>
                <h3 className="font-semibold">{provider.name}</h3>
                <p className="text-sm text-gray-500">{provider.api_base}</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-sm text-gray-400 mr-4">
                {provider.models.length} 个模型
              </span>
              <button
                onClick={() => handleEdit(provider)}
                className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
              >
                <Pencil className="w-4 h-4" />
              </button>
              <button
                onClick={() => handleDelete(provider.id)}
                className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          </div>
        ))}

        {providers.length === 0 && (
          <div className="text-center py-12 text-gray-400">
            暂无服务商，点击上方按钮添加
          </div>
        )}
      </div>

      {/* Form Modal */}
      {showForm && (
        <ProviderForm
          provider={editing || emptyProvider}
          isNew={!editing}
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

function ProviderForm({
  provider,
  isNew,
  onSave,
  onClose,
}: {
  provider: Provider;
  isNew: boolean;
  onSave: (p: Provider) => void;
  onClose: () => void;
}) {
  const [form, setForm] = useState(provider);
  const [modelsText, setModelsText] = useState(provider.models.join(", "));

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave({
      ...form,
      models: modelsText.split(",").map((m) => m.trim()).filter(Boolean),
    });
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl w-full max-w-lg p-6">
        <h2 className="text-xl font-bold mb-4">
          {isNew ? "添加服务商" : "编辑服务商"}
        </h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              名称
            </label>
            <input
              type="text"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              API Base URL
            </label>
            <input
              type="url"
              value={form.api_base}
              onChange={(e) => setForm({ ...form, api_base: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              placeholder="https://api.openai.com/v1"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              API Key
            </label>
            <input
              type="password"
              value={form.api_key}
              onChange={(e) => setForm({ ...form, api_key: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              模型列表 (逗号分隔)
            </label>
            <textarea
              value={modelsText}
              onChange={(e) => setModelsText(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              rows={3}
              placeholder="gpt-4, gpt-3.5-turbo, claude-3-opus"
            />
          </div>
          <div className="flex items-center">
            <input
              type="checkbox"
              id="enabled"
              checked={form.enabled}
              onChange={(e) => setForm({ ...form, enabled: e.target.checked })}
              className="w-4 h-4 text-primary-500 border-gray-300 rounded focus:ring-primary-500"
            />
            <label htmlFor="enabled" className="ml-2 text-sm text-gray-700">
              启用此服务商
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
