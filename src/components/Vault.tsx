import { useState, useEffect } from "react";
import { commands, type VaultEntry } from "../hooks/useTauri";
import { SearchBar } from "./SearchBar";
import { PasswordCard } from "./PasswordCard";

export function Vault() {
  const [entries, setEntries] = useState<VaultEntry[]>([]);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadEntries();
  }, []);

  async function loadEntries() {
    try {
      const list = await commands.listEntries();
      setEntries(list);
    } catch (e) {
      console.error("Failed to load entries:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(id: number) {
    try {
      await commands.deleteEntry(id);
      setEntries((prev) => prev.filter((e) => e.id !== id));
    } catch (e) {
      console.error("Delete failed:", e);
    }
  }

  const filtered = entries.filter(
    (e) =>
      e.website.toLowerCase().includes(search.toLowerCase()) ||
      e.username.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="w-full max-w-[560px]">
      <SearchBar value={search} onChange={setSearch} count={filtered.length} />

      {loading ? (
        <p className="text-[#a0a0b8] text-sm text-center py-12">Loading...</p>
      ) : filtered.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-[#a0a0b8] text-sm">
            {search ? "No matching entries" : "No saved passwords"}
          </p>
          {!search && (
            <p className="text-[#666] text-xs mt-2">
              Generate and save a password to get started
            </p>
          )}
        </div>
      ) : (
        <div className="space-y-2">
          {filtered.map((entry) => (
            <PasswordCard
              key={entry.id}
              entry={entry}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}
    </div>
  );
}
