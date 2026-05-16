import { useState, useMemo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Generator } from "./components/Generator";
import { Vault } from "./components/Vault";

type Screen = "generator" | "vault";

const isAndroid = typeof navigator !== "undefined" && /android/i.test(navigator.userAgent);

function App() {
  const [screen, setScreen] = useState<Screen>("generator");
  const [refreshKey, setRefreshKey] = useState(0);
  const appWindow = useMemo(() => getCurrentWindow(), []);

  const refreshVault = () => setRefreshKey((k) => k + 1);

  const topBarClass = isAndroid
    ? "pt-[env(safe-area-inset-top,24px)] pb-2 backdrop-blur-2xl bg-[rgba(10,10,16,0.7)]"
    : "h-10 cursor-grab active:cursor-grabbing";

  return (
    <main className="relative min-h-screen flex flex-col items-center px-4 py-8 overflow-hidden">
      {/* Title bar — draggable on desktop, blurred status bar on Android */}
      <div
        className={`fixed top-0 left-0 right-0 flex items-center justify-between px-3 z-20 ${topBarClass}`}
        data-tauri-drag-region
      >
        <span className="text-[#a0a0b8] text-xs font-bold ml-2 select-none">
          {isAndroid ? "" : "Pashword"}
        </span>
        {!isAndroid && (
          <div className="flex gap-1">
            <button
              onClick={() => appWindow.minimize()}
              className="w-10 h-7 flex items-center justify-center rounded-md text-[#a0a0b8] hover:text-white hover:bg-[rgba(255,255,255,0.08)] transition-colors text-lg font-bold leading-none cursor-pointer"
              tabIndex={-1}
            >
              &#x2013;
            </button>
            <button
              onClick={() => appWindow.close()}
              className="w-10 h-7 flex items-center justify-center rounded-md text-[#a0a0b8] hover:text-white hover:bg-red-500/60 transition-colors text-base font-bold leading-none cursor-pointer"
              tabIndex={-1}
            >
              &#x2715;
            </button>
          </div>
        )}
      </div>

      {/* Animated background */}
      <div className="fixed inset-0 bg-gradient-to-br from-[#0a0a10] via-[#0d0d1a] to-[#0a0a18] -z-10" />
      <div className="fixed inset-0 bg-[radial-gradient(ellipse_at_top,rgba(139,92,246,0.12),transparent_50%)] animate-pulse -z-10" style={{ animationDuration: "8s" }} />
      <div className="fixed top-1/4 left-1/4 w-96 h-96 bg-purple-500/10 rounded-full blur-[128px] animate-pulse -z-10" style={{ animationDuration: "6s" }} />
      <div className="fixed bottom-1/4 right-1/4 w-64 h-64 bg-fuchsia-500/10 rounded-full blur-[96px] animate-pulse -z-10" style={{ animationDuration: "10s" }} />

      {/* Nav tabs */}
      <nav className={`flex gap-2 mb-8 bg-[rgba(255,255,255,0.03)] border border-[rgba(255,255,255,0.08)] rounded-xl p-1 backdrop-blur-xl z-10 ${isAndroid ? "mt-12" : "mt-6"}`}>
        <button
          onClick={() => setScreen("generator")}
          className={`px-5 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer ${
            screen === "generator"
              ? "bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white shadow-lg shadow-purple-500/20"
              : "text-[#a0a0b8] hover:text-white"
          }`}
        >
          Generator
        </button>
        <button
          onClick={() => setScreen("vault")}
          className={`px-5 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer ${
            screen === "vault"
              ? "bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white shadow-lg shadow-purple-500/20"
              : "text-[#a0a0b8] hover:text-white"
          }`}
        >
          Vault
        </button>
      </nav>

      <div className="z-10 w-full flex justify-center">
        {screen === "generator" ? (
          <Generator onSaved={refreshVault} />
        ) : (
          <Vault key={refreshKey} />
        )}
      </div>
    </main>
  );
}

export default App;
