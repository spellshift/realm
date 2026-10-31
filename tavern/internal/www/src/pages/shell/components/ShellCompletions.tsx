import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Info } from "lucide-react";
import docsData from "../../../assets/eldritch-docs.json";
import { DocTooltip } from "./DocTooltip";
import { stdlibRegistry } from "../utils/eldritchLsp";

const docs = docsData as Record<string, { signature: string; description: string }>;

// Helper to find doc key
const findDoc = (completion: string) => {
  const cleanName = completion.replace(/[()=]/g, "").trim();

  // User-defined functions
  if (stdlibRegistry.userFunctions.has(cleanName)) {
    const fn = stdlibRegistry.userFunctions.get(cleanName)!;
    return {
      signature: fn.signature,
      description: fn.description || "User-defined function",
    };
  }

  // User-defined variables
  if (stdlibRegistry.userVariables.has(cleanName)) {
    const v = stdlibRegistry.userVariables.get(cleanName)!;
    return {
      signature: `${cleanName}: ${v.inferredType || "Any"}`,
      description: v.inferredType
        ? `User-defined variable (${v.inferredType})`
        : "User-defined variable",
    };
  }

  // 1. Exact match (e.g. "agent")
  if (docs[completion]) return docs[completion];
  if (docs[cleanName]) return docs[cleanName];

  // 2. Suffix match (e.g. "config" -> "agent.config")
  const suffix = "." + cleanName;
  const key = Object.keys(docs).find(k => k.endsWith(suffix));
  if (key) return docs[key];
  return null;
};

interface ShellCompletionsProps {
  completions: string[];
  show: boolean;
  pos: { x: number; y: number };
  index: number;
  fontSize?: number;
  onCompletionSelect?: (completion: string) => void;
}

const ShellCompletions: React.FC<ShellCompletionsProps> = ({
  completions,
  show,
  pos,
  index,
  fontSize = 13.5,
  onCompletionSelect,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const completionsListRef = useRef<HTMLUListElement>(null);
  const [hoveredDoc, setHoveredDoc] = useState<{ sig: string; desc: string; x: number; y: number } | null>(null);
  const hoverTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [adjustedPos, setAdjustedPos] = useState<{ x: number; y: number }>(pos);
  const [maxHeightStyle, setMaxHeightStyle] = useState<number | undefined>(undefined);

  useEffect(() => {
    setHoveredDoc(null);
  }, [show, completions]);

  useEffect(() => {
    if (show && completionsListRef.current) {
      const activeElement = completionsListRef.current.children[index] as HTMLElement;
      if (activeElement && typeof activeElement.scrollIntoView === "function") {
        activeElement.scrollIntoView({ block: "nearest" });
      }
    }
  }, [index, show]);

  useLayoutEffect(() => {
    if (!show) {
      setAdjustedPos(pos);
      setMaxHeightStyle(undefined);
      return;
    }

    const updatePosition = () => {
      const menuEl = containerRef.current;
      if (!menuEl) return;

      const menuRect = menuEl.getBoundingClientRect();
      const parentEl = menuEl.parentElement;
      const parentRect = parentEl?.getBoundingClientRect();

      const estimatedHeight = Math.min(200, completions.length * 26 + 8);
      const menuHeight = menuEl.offsetHeight || menuRect.height || estimatedHeight;
      const menuWidth = menuEl.offsetWidth || menuRect.width || 180;

      const parentHeight = parentEl?.clientHeight || 0;
      const parentWidth = parentEl?.clientWidth || 0;

      const boundaryHeight = parentHeight > 0 ? parentHeight : window.innerHeight;
      const boundaryWidth = parentWidth > 0 ? parentWidth : window.innerWidth;

      let newX = pos.x;
      let newY = pos.y;

      const cellHeight = (fontSize || 13.5) * 1.35;
      const promptLineGap = cellHeight + 20;

      // Check collision with the bottom boundary (Eldritch composer, drawers, or container edge)
      const isCollidingBottom =
        (boundaryHeight > 0 && pos.y + menuHeight + 8 > boundaryHeight) ||
        (parentRect && parentRect.bottom > 0 && menuRect.bottom > parentRect.bottom - 4) ||
        (window.innerHeight > 0 && menuRect.bottom > window.innerHeight - 8);

      if (isCollidingBottom) {
        // Shift up above the cursor prompt line so it doesn't collide with drawers below
        const shiftedUpY = pos.y - promptLineGap - menuHeight;
        if (shiftedUpY >= 4) {
          newY = shiftedUpY;
        } else if (boundaryHeight > 0) {
          newY = Math.max(4, boundaryHeight - menuHeight - 8);
        }
      }

      // Check collision with the right boundary (side drawers or container edge)
      const isCollidingRight =
        (boundaryWidth > 0 && pos.x + menuWidth + 8 > boundaryWidth) ||
        (parentRect && parentRect.right > 0 && menuRect.right > parentRect.right - 4) ||
        (window.innerWidth > 0 && menuRect.right > window.innerWidth - 8);

      if (isCollidingRight && boundaryWidth > 0) {
        newX = Math.max(4, boundaryWidth - menuWidth - 8);
      }

      const calculatedMaxHeight =
        boundaryHeight > 0 && boundaryHeight < 220
          ? Math.min(200, Math.max(60, boundaryHeight - 16))
          : undefined;

      setMaxHeightStyle(calculatedMaxHeight);
      setAdjustedPos((prev) => {
        if (prev.x === newX && prev.y === newY) return prev;
        return { x: newX, y: newY };
      });
    };

    updatePosition();
    window.addEventListener("resize", updatePosition);

    let resizeObserver: ResizeObserver | null = null;
    if (typeof ResizeObserver !== "undefined" && containerRef.current?.parentElement) {
      try {
        resizeObserver = new ResizeObserver(() => {
          updatePosition();
        });
        resizeObserver.observe(containerRef.current.parentElement);
      } catch {
        // Ignore ResizeObserver errors in test environments
      }
    }

    return () => {
      window.removeEventListener("resize", updatePosition);
      resizeObserver?.disconnect();
    };
  }, [show, pos, completions.length, fontSize]);

  if (!show) return null;

  return (
    <>
      <div
        ref={containerRef}
        data-testid="shell-completions"
        style={{
          top: adjustedPos.y,
          left: adjustedPos.x,
          ...(maxHeightStyle ? { maxHeight: `${maxHeightStyle}px` } : {}),
        }}
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
