import { useState } from "react";
import { commands } from "../hooks/useTauri";

interface Props {
  initialized: boolean;
  onUnlocked: () => void;
}

export function UnlockScreen({ initialized, onUnlocked }: Props) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const isSetup = !initialized;

  async function handleSubmit() {
    setError("");
    if (isSetup && password !== confirm) {
      setError("Passwords don't match");
      return;
    }
    if (password.length < 4) {
      setError("Password must be at least 4 characters");
      return;
    }

    setLoading(true);
    try {
      if (isSetup) {
        await commands.setupVault(password);
        onUnlocked();
      } else {
        const ok = await commands.unlockVault(password);
        if (ok) {
          onUnlocked();
        } else {
          setError("Incorrect master password");
        }
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="relative min-h-screen flex flex-col items-center justify-center px-4">
      <div className="fixed inset-0 bg-gradient-to-br from-[#0a0a10] via-[#0d0d1a] to-[#0a0a18] -z-10" />
      <div className="fixed inset-0 bg-[radial-gradient(ellipse_at_top,rgba(139,92,246,0.08),transparent_50%)] -z-10" />

      <div className="w-full max-w-[420px] bg-[rgba(255,255,255,0.03)] border border-[rgba(255,255,255,0.08)] rounded-2xl p-8 backdrop-blur-xl shadow-2xl">
        <h1 className="text-2xl font-bold text-white text-center mb-1">Pashword</h1>
        <p className="text-[#a0a0b8] text-sm text-center mb-6">
          {isSetup ? "Create your master password" : "Unlock your vault"}
        </p>

        <div className="space-y-4">
          <input
            type="password"
            placeholder="Master password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
            className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
          />

          {isSetup && (
            <input
              type="password"
              placeholder="Confirm master password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
              className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
            />
          )}

          {error && (
            <p className="text-red-400 text-xs text-center">{error}</p>
          )}

          <button
            onClick={handleSubmit}
            disabled={loading}
            className="w-full bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white font-semibold rounded-[10px] py-3 text-sm shadow-lg shadow-purple-500/20 hover:brightness-110 transition-all disabled:opacity-50"
          >
            {loading ? "..." : isSetup ? "Create Vault" : "Unlock"}
          </button>

          {isSetup && (
            <p className="text-[#a0a0b8] text-xs text-center mt-4">
              If you forget this password, your vault is <strong className="text-red-400">gone forever</strong>.
              There is no recovery.
            </p>
          )}
        </div>
      </div>
    </main>
  );
}
