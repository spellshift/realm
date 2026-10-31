import React, { useEffect, useRef, useState } from "react";
import { Info } from "lucide-react";
import docsData from "../../../assets/eldritch-docs.json";
import { DocTooltip } from "./DocTooltip";

const docs = docsData as Record<string, { signature: string; description: string }>;

// Helper to find doc key
const findDoc = (completion: string) => {
  // 1. Exact match (e.g. "agent")
  if (docs[completion]) return docs[completion];
  // 2. Suffix match (e.g. "config" -> "agent.config")
  // This assumes no collisions or picks the first one.
  // Given eldritch DSL structure, collisions on method names are possible but likely context specific.
  // However, completions usually come with context.
  // If we only have the completion text "config", we try to find a key ending in ".config".
  const suffix = "." + completion;
  const key = Object.keys(docs).find(k => k.endsWith(suffix));
  if (key) return docs[key];
  return null;
};

interface ShellCompletionsProps {
  completions: string[];
  show: boolean;
  pos: { x: number; y: number };
  index: number;
  onCompletionSelect?: (completion: string) => void;
}

const ShellCompletions: React.FC<ShellCompletionsProps> = ({ completions, show, pos, index, onCompletionSelect }) => {
  const completionsListRef = useRef<HTMLUListElement>(null);
  const [hoveredDoc, setHoveredDoc] = useState<{ sig: string, desc: string, x: number, y: number } | null>(null);
  const hoverTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setHoveredDoc(null);
  }, [show, completions]);

  useEffect(() => {
    if (show && completionsListRef.current) {
      const activeElement = completionsListRef.current.children[index] as HTMLElement;
      if (activeElement) {
        activeElement.scrollIntoView({ block: "nearest" });
      }
    }
  }, [index, show]);

  if (!show) return null;

  return (
    <>
      <div
        style={{ top: pos.y, left: pos.x }}
        className="absolute z-[1000] max-h-[200px] overflow-y-auto rounded-md bg-gray-900 border border-gray-700 shadow-lg text-gray-200 font-mono text-xs"
      >
        <ul ref={completionsListRef} className="list-none m-0 p-0.5">
          {completions.map((c, i) => {
            const doc = findDoc(c);
            const isSelected = i === index;
            const rowClass = isSelected
              ? "bg-purple-600/30 text-purple-100"
              : "hover:bg-gray-800 text-gray-300";

            return (
              <li
                key={i}
                className={`px-2 py-1 rounded cursor-pointer flex items-center justify-between transition-colors ${rowClass}`}
                onMouseDown={(e) => {
                  e.preventDefault();
                  if (onCompletionSelect) {
                    onCompletionSelect(c);
                  }
                }}
                onMouseEnter={(e) => {
                  if (hoverTimeoutRef.current) {
                    clearTimeout(hoverTimeoutRef.current);
                    hoverTimeoutRef.current = null;
                  }
                  if (doc) {
                    const rect = e.currentTarget.getBoundingClientRect();
                    setHoveredDoc({
                      sig: doc.signature,
                      desc: doc.description,
                      x: rect.right + 10,
                      y: rect.top
                    });
                  } else {
                    setHoveredDoc(null);
                  }
                }}
                onMouseLeave={() => {
                  if (hoverTimeoutRef.current) clearTimeout(hoverTimeoutRef.current);
                  hoverTimeoutRef.current = setTimeout(() => {
                    setHoveredDoc(null);
                  }, 150);
                }}
              >
                <span>{c}</span>
                {doc && <Info size={14} className="ml-2 text-gray-400" />}
              </li>
            );
          })}
        </ul>
      </div>
      {hoveredDoc && (
        <DocTooltip
          signature={hoveredDoc.sig}
          description={hoveredDoc.desc}
          x={hoveredDoc.x}
          y={hoveredDoc.y}
          visible={true}
          onMouseEnter={() => {
            if (hoverTimeoutRef.current) {
              clearTimeout(hoverTimeoutRef.current);
              hoverTimeoutRef.current = null;
            }
          }}
          onMouseLeave={() => {
            if (hoverTimeoutRef.current) clearTimeout(hoverTimeoutRef.current);
            hoverTimeoutRef.current = setTimeout(() => {
              setHoveredDoc(null);
            }, 250);
          }}
        />
      )}
    </>
  );
};

export default ShellCompletions;
