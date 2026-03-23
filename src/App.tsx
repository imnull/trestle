import { useState, useEffect } from "react";
import { Sidebar } from "./components/Sidebar";
import { Dashboard } from "./pages/Dashboard";
import { Providers } from "./pages/Providers";
import { Routes } from "./pages/Routes";
import { Logs } from "./pages/Logs";
import { Settings } from "./pages/Settings";
import { useStore } from "./stores/app";

function App() {
  const { currentPage, init } = useStore();

  useEffect(() => {
    init();
  }, [init]);

  const renderPage = () => {
    switch (currentPage) {
      case "dashboard":
        return <Dashboard />;
      case "providers":
        return <Providers />;
      case "routes":
        return <Routes />;
      case "logs":
        return <Logs />;
      case "settings":
        return <Settings />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="flex h-screen bg-gray-50">
      <Sidebar />
      <main className="flex-1 overflow-auto">
        {renderPage()}
      </main>
    </div>
  );
}

export default App;
