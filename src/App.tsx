import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useEffect } from "react";
import { BrowserRouter, Navigate, Outlet, Route, Routes } from "react-router-dom";
import { Sidebar } from "./components/Sidebar";
import { Toaster } from "./components/ui/toaster";
import { useThemeStore } from "./lib/theme-store";
import { Dashboard } from "./pages/Dashboard";
import { History } from "./pages/History";
import { Orphans } from "./pages/Orphans";
import { Packages } from "./pages/Packages";

const queryClient = new QueryClient();

function ThemeSync(): null {
  const mode = useThemeStore((state) => state.mode);

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", mode === "dark");
  }, [mode]);

  return null;
}

function AppLayout() {
  return (
    <div className="min-h-screen bg-muted/30">
      <Sidebar />
      <main className="ml-[220px] min-h-screen p-4 md:p-6">
        <Outlet />
      </main>
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeSync />
      <BrowserRouter>
        <Routes>
          <Route element={<AppLayout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/packages" element={<Packages />} />
            <Route path="/orphans" element={<Orphans />} />
            <Route path="/history" element={<History />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
        </Routes>
      </BrowserRouter>
      <Toaster />
    </QueryClientProvider>
  );
}

export default App;
