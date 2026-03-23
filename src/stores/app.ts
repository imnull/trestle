import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

// ============ Types ============

export interface ServerStatus {
  running: boolean;
  port: number;
  uptime_secs: number;
  active_connections: number;
}

export interface Provider {
  id: string;
  name: string;
  api_base: string;
  api_key: string;
  models: string[];
  enabled: boolean;
}

export interface Route {
  id: string;
  path: string;
  target_provider: string;
  target_model: string;
  enabled: boolean;
}

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  request_id?: string;
}

type Page = "dashboard" | "providers" | "routes" | "logs" | "settings";

// ============ Store ============

interface AppState {
  currentPage: Page;
  serverStatus: ServerStatus | null;
  providers: Provider[];
  routes: Route[];
  logs: LogEntry[];
  connected: boolean;
  loading: boolean;

  // Actions
  setCurrentPage: (page: Page) => void;
  setServerStatus: (status: ServerStatus | null) => void;
  setProviders: (providers: Provider[]) => void;
  setRoutes: (routes: Route[]) => void;
  setLogs: (logs: LogEntry[]) => void;
  setConnected: (connected: boolean) => void;

  // API calls
  init: () => void;
  fetchStatus: () => Promise<void>;
  fetchProviders: () => Promise<void>;
  fetchRoutes: () => Promise<void>;
  fetchLogs: () => Promise<void>;

  // Mutations
  addProvider: (provider: Provider) => Promise<void>;
  updateProvider: (id: string, provider: Provider) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
  addRoute: (route: Route) => Promise<void>;
  updateRoute: (id: string, route: Route) => Promise<void>;
  deleteRoute: (id: string) => Promise<void>;
}

export const useStore = create<AppState>((set, get) => ({
  currentPage: "dashboard",
  serverStatus: null,
  providers: [],
  routes: [],
  logs: [],
  connected: false,
  loading: false,

  setCurrentPage: (page) => set({ currentPage: page }),
  setServerStatus: (status) => set({ serverStatus: status, connected: !!status }),
  setProviders: (providers) => set({ providers }),
  setRoutes: (routes) => set({ routes }),
  setLogs: (logs) => set({ logs }),
  setConnected: (connected) => set({ connected }),

  init: () => {
    const { fetchStatus, fetchProviders, fetchRoutes, fetchLogs } = get();
    
    // 初始加载
    fetchStatus();
    fetchProviders();
    fetchRoutes();
    fetchLogs();

    // 定期刷新状态
    setInterval(() => {
      get().fetchStatus();
    }, 5000);
  },

  fetchStatus: async () => {
    try {
      const status = await invoke<ServerStatus>("get_server_status");
      set({ serverStatus: status, connected: true });
    } catch (e) {
      set({ serverStatus: null, connected: false });
    }
  },

  fetchProviders: async () => {
    try {
      const providers = await invoke<Provider[]>("get_providers");
      set({ providers });
    } catch (e) {
      console.error("Failed to fetch providers:", e);
    }
  },

  fetchRoutes: async () => {
    try {
      const routes = await invoke<Route[]>("get_routes");
      set({ routes });
    } catch (e) {
      console.error("Failed to fetch routes:", e);
    }
  },

  fetchLogs: async () => {
    try {
      const logs = await invoke<LogEntry[]>("get_logs", { limit: 100 });
      set({ logs });
    } catch (e) {
      console.error("Failed to fetch logs:", e);
    }
  },

  addProvider: async (provider) => {
    await invoke("add_provider", { provider });
    get().fetchProviders();
  },

  updateProvider: async (id, provider) => {
    await invoke("update_provider", { id, provider });
    get().fetchProviders();
  },

  deleteProvider: async (id) => {
    await invoke("delete_provider", { id });
    get().fetchProviders();
  },

  addRoute: async (route) => {
    await invoke("add_route", { route });
    get().fetchRoutes();
  },

  updateRoute: async (id, route) => {
    await invoke("update_route", { id, route });
    get().fetchRoutes();
  },

  deleteRoute: async (id) => {
    await invoke("delete_route", { id });
    get().fetchRoutes();
  },
}));
