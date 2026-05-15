interface Props {
  value: string;
  onChange: (v: string) => void;
  count: number;
}

export function SearchBar({ value, onChange, count }: Props) {
  return (
    <div className="flex items-center gap-3 mb-4">
      <div className="relative flex-1">
        <input
          type="text"
          placeholder="Search vault..."
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="w-full bg-[#0d0d18] border border-[rgba(255,255,255,0.10)] rounded-[10px] px-4 py-2.5 text-white placeholder-[#666] text-sm focus:outline-none focus:border-[#8b5cf6] focus:shadow-[0_0_0_2px_rgba(139,92,246,0.3)] transition-all"
        />
        {value && (
          <button
            onClick={() => onChange("")}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-[#666] hover:text-white text-sm"
            type="button"
          >
            Clear
          </button>
        )}
      </div>
      <span className="text-xs text-[#a0a0b8] whitespace-nowrap">
        {count} {count === 1 ? "entry" : "entries"}
      </span>
    </div>
  );
}
