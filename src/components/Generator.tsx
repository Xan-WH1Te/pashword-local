import { useState } from "react";
import { commands } from "../hooks/useTauri";

interface Props {
  onSaved: () => void;
}

export function Generator({ onSaved }: Props) {
  const [website, setWebsite] = useState("");
  const [username, setUsername] = useState("");
  const [secretKey, setSecretKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [length, setLength] = useState(20);
  const [generated, setGenerated] = useState("");
  const [copied, setCopied] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveMsg, setSaveMsg] = useState("");

  function copyToClipboard(text: string, label: string) {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(label);
      setTimeout(() => setCopied(null), 2000);
    });
  }

  async function handleGenerate() {
    if (!website || !username || !secretKey) return;
    try {
      const pw = await commands.generatePassword(website, username, secretKey, length);
      setGenerated(pw);
    } catch (e) {
      setGenerated("Error: " + String(e));
    }
  }

  async function handleSave() {
    if (!generated) return;
    setSaving(true);
    setSaveMsg("");
    try {
      await commands.saveEntry(website, username, secretKey, generated);
      setSaveMsg("Saved to vault!");
      onSaved();
      setTimeout(() => setSaveMsg(""), 2000);
    } catch (e) {
      setSaveMsg("Save failed: " + String(e));
    } finally {
      setSaving(false);
    }
  }

  const canGenerate = website && username && secretKey;

  const LengthLabel: Record<number, string> = {
    11: "Short",
    15: "Medium",
    20: "Large (Recommended)",
    24: "Long",
    32: "Extra Long",
    40: "Huge",
  };

  const lengthOptions = [11, 15, 20, 24, 32, 40, 48, 64];

  return (
    <div className="w-full max-w-[500px] bg-[rgba(255,255,255,0.04)] border border-[rgba(255,255,255,0.08)] rounded-2xl p-8 backdrop-blur-2xl shadow-2xl" style={{ boxShadow: "0 8px 32px rgba(139,92,246,0.08), 0 0 0 1px rgba(255,255,255,0.04) inset" }}>
      <h2 className="text-xl font-bold text-white text-center mb-1">
        Pashword
      </h2>
      <p className="text-[#a0a0b8] text-sm text-center mb-6">
        Passwords done right
      </p>

      <div className="space-y-4">
        {/* Website */}
        <div>
          <label className="block text-xs text-[#a0a0b8] uppercase tracking-wider mb-1.5">
            Website{" "}
            <span className="text-[#666] font-normal normal-case tracking-normal">
              — Enter the website address here
            </span>
          </label>
          <input
            type="text"
            placeholder="Example: reddit.com, forum.zorin.com"
            value={website}
            onChange={(e) => setWebsite(e.target.value)}
            className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white placeholder-[#555] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
          />
        </div>

        {/* Username */}
        <div>
          <label className="block text-xs text-[#a0a0b8] uppercase tracking-wider mb-1.5">
            Username{" "}
            <span className="text-[#666] font-normal normal-case tracking-normal">
              — Enter the username for that website
            </span>
          </label>
          <input
            type="text"
            placeholder="Example: nayam_amarshe"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white placeholder-[#555] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
          />
        </div>

        {/* Secret Key */}
        <div>
          <label className="block text-xs text-[#a0a0b8] uppercase tracking-wider mb-1.5">
            Secret Key{" "}
            <span className="text-[#666] font-normal normal-case tracking-normal">
              — A strong secret only you know. Keep it safe and use it every time.
            </span>
          </label>
          <div className="relative">
            <input
              type={showKey ? "text" : "password"}
              placeholder="Example: JimmyNeutron10$"
              value={secretKey}
              onChange={(e) => setSecretKey(e.target.value)}
              className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 pr-14 text-white placeholder-[#555] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
            />
            <button
              onClick={() => setShowKey(!showKey)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-[#666] hover:text-[#a0a0b8] text-xs transition-colors"
              tabIndex={-1}
              type="button"
            >
              {showKey ? "Hide" : "Show"}
            </button>
          </div>
        </div>

        {/* Pashword Length */}
        <div>
          <label className="block text-xs text-[#a0a0b8] uppercase tracking-wider mb-1.5">
            Pashword Length{" "}
            <span className="text-[#666] font-normal normal-case tracking-normal">
              — Change if the website has specific requirements
            </span>
          </label>
          <select
            value={length}
            onChange={(e) => setLength(Number(e.target.value))}
            className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-3 text-white text-sm focus:outline-none focus:border-[#8b5cf6] transition-all appearance-none cursor-pointer"
          >
            {lengthOptions.map((n) => (
              <option key={n} value={n}>
                {LengthLabel[n] ? `${n} — ${LengthLabel[n]}` : n}
              </option>
            ))}
          </select>
        </div>

        {/* Generate Button */}
        <button
          onClick={handleGenerate}
          disabled={!canGenerate}
          className="w-full bg-gradient-to-r from-[#8b5cf6] to-[#a855f7] text-white font-semibold rounded-[10px] py-3 text-sm shadow-lg shadow-purple-500/20 hover:brightness-110 transition-all disabled:opacity-40 disabled:cursor-not-allowed"
        >
          Get Pashword 😎
        </button>

        {/* Result */}
        {generated && (
          <div className="mt-4 p-4 bg-[rgba(255,255,255,0.03)] border border-[rgba(255,255,255,0.08)] rounded-[10px] space-y-3 backdrop-blur-xl">
            <div className="flex items-center justify-between">
              <span className="text-xs text-[#a0a0b8] uppercase tracking-wider">
                Your Pashword
              </span>
              <button
                onClick={() => copyToClipboard(generated, "password")}
                className="text-xs text-[#8b5cf6] hover:text-[#a855f7] transition-colors"
                type="button"
              >
                {copied === "password" ? "Copied!" : "Copy"}
              </button>
            </div>
            <p className="text-white text-sm break-all font-mono bg-[#0a0a10] rounded-lg p-3">
              {generated}
            </p>

            <button
              onClick={handleSave}
              disabled={saving}
              className="w-full bg-[rgba(139,92,246,0.15)] border border-[rgba(139,92,246,0.3)] text-[#a78bfa] font-medium rounded-[10px] py-2.5 text-sm hover:bg-[rgba(139,92,246,0.25)] transition-all disabled:opacity-50"
              type="button"
            >
              {saving ? "Saving..." : "Save to Vault"}
            </button>
            {saveMsg && (
              <p className={`text-xs text-center ${saveMsg.includes("failed") ? "text-red-400" : "text-green-400"}`}>
                {saveMsg}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
