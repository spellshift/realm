import React, { useEffect, useRef, useState } from "react";
import { Info } from "lucide-react";
import docsData from "../../../assets/eldritch-docs.json";
import { DocTooltip } from "./DocTooltip";

const docs = docsData as Record<string, { signature: string; description: string }>;

// Helper to find doc key
const findDoc = (completion: string) => {
    // 1. Exact match (e.g. "agent")
    if (docs[completion]) return docs[completion];
    // 2. Suffix match (e.g. "get_config" -> "agent.get_config")
    // This assumes no collisions or picks the first one.
    // Given eldritch DSL structure, collisions on method names are possible but likely context specific.
    // However, completions usually come with context.
    // If we only have the completion text "get_config", we try to find a key ending in ".get_config".
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
  onSelect: (completion: string) => void;
}

const ShellCompletions: React.FC<ShellCompletionsProps> = ({ completions, show, pos, index, onSelect }) => {
  const completionsListRef = useRef<HTMLUListElement>(null);
  const [hoveredDoc, setHoveredDoc] = useState<{ sig: string, desc: string, x: number, y: number } | null>(null);

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
      <div style={{
        position: "absolute",
        top: pos.y,
        left: pos.x,
        background: "#252526",
        border: "1px solid #454545",
        zIndex: 1000,
        maxHeight: "200px",
        overflowY: "auto",
        boxShadow: "0 4px 6px rgba(0,0,0,0.3)",
        color: "#cccccc",
        fontFamily: 'Menlo, Monaco, "Courier New", monospace',
        fontSize: "14px"
      }}>
        <ul ref={completionsListRef} style={{ listStyle: "none", margin: 0, padding: 0 }}>
          {completions.map((c, i) => {
            const doc = findDoc(c);
            return (
                <li
                key={i}
                style={{
                    padding: "4px 8px",
                    background: i === index ? "#094771" : "transparent",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between"
                }}
                onClick={() => onSelect(c)}
                onMouseDown={(e) => e.preventDefault()}
                onMouseEnter={(e) => {
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
                onMouseLeave={() => setHoveredDoc(null)}
                >
                <span>{c}</span>
                {doc && <Info size={14} style={{ marginLeft: 8 }} />}
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
          />
      )}
    </>
  );
};

export default ShellCompletions;
