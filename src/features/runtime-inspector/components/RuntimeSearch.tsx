import { useRef } from "react";

export function RuntimeSearch({
  query,
  onQueryChange,
}: {
  query: string;
  onQueryChange: (query: string) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <div className="runtime-search-row">
      <div className="runtime-search">
        <label className="sr-only" htmlFor="runtime-search-input">
          Search listeners by process or port
        </label>
        <svg
          className="runtime-search-icon"
          viewBox="0 0 20 20"
          aria-hidden="true"
          focusable="false"
        >
          <circle cx="8.5" cy="8.5" r="5.5" />
          <path d="m12.5 12.5 4 4" />
        </svg>
        <input
          ref={inputRef}
          id="runtime-search-input"
          type="search"
          value={query}
          onChange={(event) => onQueryChange(event.currentTarget.value)}
          placeholder="Search by process or port"
          autoComplete="off"
          spellCheck={false}
        />
        {query.length > 0 && (
          <button
            className="runtime-search-clear"
            type="button"
            aria-label="Clear search"
            onClick={() => {
              onQueryChange("");
              inputRef.current?.focus();
            }}
          >
            <span aria-hidden="true">×</span>
          </button>
        )}
      </div>
    </div>
  );
}
