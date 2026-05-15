import { useState, useEffect } from "react";
import { commands } from "./hooks/useTauri";
import { UnlockScreen } from "./components/UnlockScreen";
import { Generator } from "./components/Generator";
import { Vault } from "./components/Vault";

type Screen = "generator" | "vault";

function App() {
  const [unlocked, setUnlocked] = useState(false);
  const [initialized, setInitialized] = useState<boolean | null>(null);
  const [screen, setScreen] = useState<Screen>("generator");
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    commands.isVaultInitialized().then(setInitialized);
  }, []);

  if (initialized === null) return null;

  if (!unlocked) {
    return (
      <UnlockScreen
        initialized={initialized}
        onUnlocked={() => setUnlocked(true)}
      />
    );
  }

  const refreshVault = () => setRefreshKey((k) => k + 1);

  return (
    <main className="relative min-h-screen flex flex-col items-center px-4 py-8">
      {/* Background gradient */}
      <div className="fixed inset-0 bg-gradient-to-br from-[#0a0a10] via-[#0d0d1a] to-[#0a0a18] -z-10" />
      <div className="fixed inset-0 bg-[radial-gradient(ellipse_at_top,rgba(139,92,246,0.08),transparent_50%)] -z-10" />

      {/* Nav tabs */}
      <nav className="flex gap-2 mb-8 bg-[rgba(255,255,255,0.03)] border border-[rgba(255,255,255,0.08)] rounded-xl p-1 backdrop-blur-xl">
        <button
          onClick={() => setScreen("generator")}
          className={`px-5 py-2 rounded-lg text-sm font-medium transition-all ${
            screen === "generator"
              ? "bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white shadow-lg shadow-purple-500/20"
              : "text-[#a0a0b8] hover:text-white"
          }`}
        >
          Generator
        </button>
        <button
          onClick={() => setScreen("vault")}
          className={`px-5 py-2 rounded-lg text-sm font-medium transition-all ${
            screen === "vault"
              ? "bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white shadow-lg shadow-purple-500/20"
              : "text-[#a0a0b8] hover:text-white"
          }`}
        >
          Vault
        </button>
      </nav>

      {screen === "generator" ? (
        <Generator onSaved={refreshVault} />
      ) : (
        <Vault key={refreshKey} />
      )}
    </main>
  );
}

export default App;
