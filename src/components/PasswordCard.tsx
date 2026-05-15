import { useState } from "react";
import type { VaultEntry } from "../hooks/useTauri";

interface Props {
  entry: VaultEntry;
  onDelete: (id: number) => void;
}

export function PasswordCard({ entry, onDelete }: Props) {
  const [expanded, setExpanded] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);

  function copyToClipboard(text: string, label: string) {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(label);
      setTimeout(() => setCopied(null), 2000);
    });
  }

  function handleDelete() {
    if (confirm(`Delete entry for ${entry.website}?`)) {
      onDelete(entry.id);
    }
  }

  return (
    <div className="bg-[rgba(255,255,255,0.02)] border border-[rgba(255,255,255,0.06)] rounded-xl overflow-hidden transition-all hover:border-[rgba(255,255,255,0.12)]">
      {/* Collapsed row */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between px-4 py-3.5 text-left"
      >
        <div className="min-w-0">
          <p className="text-white text-sm font-medium truncate">
            {entry.website}
          </p>
          <p className="text-[#a0a0b8] text-xs truncate">{entry.username}</p>
        </div>
        <span className="text-[#666] text-xs ml-3 shrink-0">
          {expanded ? "▲" : "▼"}
        </span>
      </button>

      {/* Expanded details */}
      {expanded && (
        <div className="px-4 pb-4 space-y-3 border-t border-[rgba(255,255,255,0.06)] pt-3">
          <FieldRow
            label="Website"
            value={entry.website}
            copied={copied}
            onCopy={() => copyToClipboard(entry.website, "w-" + entry.id)}
            copyId={"w-" + entry.id}
          />
          <FieldRow
            label="Username"
            value={entry.username}
            copied={copied}
            onCopy={() => copyToClipboard(entry.username, "u-" + entry.id)}
            copyId={"u-" + entry.id}
          />
          <FieldRow
            label="Secret Key"
            value={entry.secret_key}
            copied={copied}
            onCopy={() => copyToClipboard(entry.secret_key, "sk-" + entry.id)}
            copyId={"sk-" + entry.id}
            sensitive
          />
          <FieldRow
            label="Pashword"
            value={entry.pashword}
            copied={copied}
            onCopy={() => copyToClipboard(entry.pashword, "pw-" + entry.id)}
            copyId={"pw-" + entry.id}
            mono
          />

          <button
            onClick={handleDelete}
            className="w-full text-center text-xs text-red-400/70 hover:text-red-400 py-2 transition-colors"
          >
            Delete Entry
          </button>
        </div>
      )}
    </div>
  );
}

function FieldRow({
  label,
  value,
  copied,
  copyId,
  onCopy,
  sensitive,
  mono,
}: {
  label: string;
  value: string;
  copied: string | null;
  copyId: string;
  onCopy: () => void;
  sensitive?: boolean;
  mono?: boolean;
}) {
  return (
    <div className="flex items-start justify-between gap-2">
      <div className="min-w-0">
        <p className="text-xs text-[#666] uppercase tracking-wider mb-0.5">
          {label}
        </p>
        <p
          className={`text-sm text-white break-all ${
            mono ? "font-mono text-xs" : ""
          } ${sensitive ? "blur-sm hover:blur-none transition-all select-none" : ""}`}
        >
          {value}
        </p>
      </div>
      <button
        onClick={onCopy}
        className="text-xs text-[#8b5cf6] hover:text-[#a855f7] shrink-0 transition-colors mt-1"
        type="button"
      >
        {copied === copyId ? "Copied!" : "Copy"}
      </button>
    </div>
  );
}
