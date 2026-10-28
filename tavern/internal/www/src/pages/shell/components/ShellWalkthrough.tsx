import React, { useEffect, useState } from "react";
import {
  ChevronLeft,
  ChevronRight,
  Code2,
  Compass,
  Database,
  History,
  Sparkles,
  Terminal,
  X,
  Zap,
} from "lucide-react";

export interface WalkthroughStep {
  id: string;
  selector: string;
  title: string;
  description: string;
  tip: string;
  icon: React.ReactNode;
}

const WALKTHROUGH_STEPS: WalkthroughStep[] = [
  {
    id: "terminal",
    selector: '[data-tour="shell-terminal"]',
    title: "Eldritch REPL & Multi-Line Editing",
    description:
      "Type single commands or multi-line blocks (def, for, if). Use Up & Down arrows to move between lines inside an active block and fix typos before pressing Enter on a trailing blank line.",
    tip: "Prefix OS commands with ! (e.g. !whoami) or press Shift+Enter for a manual newline.",
    icon: <Terminal size={16} className="text-purple-400" />,
  },
  {
    id: "portal",
    selector: '[data-tour="shell-portal"]',
    title: "Interactive Portals & Pivots",
    description:
      "Shells start in Queued Mode (commands run on next beacon check-in). Run portal.create() to switch to real-time Portal Active mode and unlock + PTY and + SSH pivot tabs.",
    tip: "SSH pivots negotiate directly from the gRPC-receiving Tavern instance over the beacon's portal.",
    icon: <Zap size={16} className="text-emerald-400" />,
  },
  {
    id: "composer",
    selector: '[data-tour="shell-composer"]',
    title: "Composer & Shell Aliases",
    description:
      "Toggle the bottom Composer to draft larger Eldritch scripts or insert 1-click templates—including the aliases snippet that registers ls(), whoami(), ps(), and netstat().",
    tip: "Press Ctrl+Enter inside the Composer to execute immediately.",
    icon: <Code2 size={16} className="text-purple-400" />,
  },
  {
    id: "knowledge",
    selector: '[data-tour="shell-knowledge"]',
    title: "Knowledge Graph",
    description:
      "Access real-time host credentials, network topology, and reported files. Download discovered host files locally in one click or use captured credentials to launch SSH pivots.",
    tip: "Polled live every 5 seconds so newly reported credentials and files appear automatically.",
    icon: <Database size={16} className="text-emerald-400" />,
  },
  {
    id: "history",
    selector: '[data-tour="shell-history"]',
    title: "Command Queue & History",
    description:
      "Inspect queued, running, and completed commands alongside all active or closed PTY/SSH pivot sessions. Filter by status or re-stage any historical command.",
    tip: "On slow-callback beacons, queue multiple commands in sequence and track their execution here.",
    icon: <History size={16} className="text-blue-400" />,
  },
  {
    id: "guide",
    selector: '[data-tour="shell-guide"]',
    title: "Operator Guide & Stdlib Reference",
    description:
      "Browse the searchable Eldritch standard library documentation, keyboard shortcuts, or replay this walkthrough anytime.",
    tip: "Click Prompt or Composer on any stdlib function to stage it directly.",
    icon: <Compass size={16} className="text-amber-400" />,
  },
];

interface ShellWalkthroughProps {
  isOpen: boolean;
  onDismiss: () => void;
  onOpenGuideDrawer?: () => void;
}

interface TargetRect {
  top: number;
  left: number;
  width: number;
  height: number;
}

const ShellWalkthrough: React.FC<ShellWalkthroughProps> = ({
  isOpen,
  onDismiss,
  onOpenGuideDrawer,
}) => {
  const [stepIndex, setStepIndex] = useState(0);
  const [targetRect, setTargetRect] = useState<TargetRect | null>(null);

  const currentStep = WALKTHROUGH_STEPS[stepIndex] || WALKTHROUGH_STEPS[0];

  useEffect(() => {
    if (!isOpen) {
      setStepIndex(0);
      return;
    }

    const updateRect = () => {
      const el = document.querySelector(currentStep.selector);
      if (!el) {
        setTargetRect(null);
        return;
      }
      const r = el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) {
        setTargetRect(null);
        return;
      }
      setTargetRect({
        top: r.top,
        left: r.left,
        width: r.width,
        height: r.height,
      });
    };

    updateRect();
    window.addEventListener("resize", updateRect);
    window.addEventListener("scroll", updateRect, true);
    return () => {
      window.removeEventListener("resize", updateRect);
      window.removeEventListener("scroll", updateRect, true);
    };
  }, [isOpen, stepIndex, currentStep.selector]);

  if (!isOpen) return null;

  const isLastStep = stepIndex === WALKTHROUGH_STEPS.length - 1;

  // Compute popover position near the highlighted element
  const getPopoverStyle = (): React.CSSProperties => {
    if (!targetRect) {
      return {
        top: "88px",
        right: "24px",
      };
    }

    const viewportW = typeof window !== "undefined" ? window.innerWidth : 1280;
    const viewportH = typeof window !== "undefined" ? window.innerHeight : 800;
    const popoverW = 360;

    // If highlighting the large terminal area, dock the popover cleanly in the top-right of the terminal
    if (currentStep.id === "terminal") {
      return {
        top: `${Math.max(76, targetRect.top + 16)}px`,
        right: "28px",
      };
    }

    const belowTop = targetRect.top + targetRect.height + 12;
    const top =
      belowTop + 230 < viewportH
        ? belowTop
        : Math.max(16, targetRect.top - 220);

    const idealLeft = targetRect.left + targetRect.width / 2 - popoverW / 2;
    const clampedLeft = Math.max(16, Math.min(idealLeft, viewportW - popoverW - 16));

    return {
      top: `${top}px`,
      left: `${clampedLeft}px`,
    };
  };

  return (
    <div
      data-testid="shell-nux-banner"
      aria-label="Shell interactive walkthrough"
      className="fixed inset-0 z-50 pointer-events-none"
    >
      {/* Spotlight Cutout Ring around the highlighted component */}
      {targetRect ? (
        <div
          data-testid="walkthrough-spotlight"
          className="fixed rounded-lg border-2 border-purple-400/90 transition-all duration-200 pointer-events-none"
          style={{
            top: `${Math.max(2, targetRect.top - 4)}px`,
            left: `${Math.max(2, targetRect.left - 4)}px`,
            width: `${targetRect.width + 8}px`,
            height: `${targetRect.height + 8}px`,
            boxShadow:
              "0 0 0 9999px rgba(7, 9, 14, 0.65), 0 0 24px rgba(168, 85, 247, 0.45)",
          }}
        />
      ) : (
        <div className="fixed inset-0 bg-black/50 pointer-events-none" />
      )}

      {/* Floating Walkthrough Popover */}
      <div
        role="dialog"
        aria-modal="false"
        aria-label={currentStep.title}
        style={getPopoverStyle()}
        className="fixed w-[350px] max-w-[calc(100vw-32px)] rounded-xl bg-[#161922] border border-purple-500/40 shadow-2xl p-4 text-xs text-gray-200 pointer-events-auto transition-all duration-200"
      >
        {/* Header */}
        <div className="flex items-center justify-between gap-2 mb-2.5">
          <div className="flex items-center gap-2 min-w-0">
            <span className="p-1.5 rounded-lg bg-purple-500/15 border border-purple-500/30 flex-shrink-0">
              {currentStep.icon}
            </span>
            <div className="min-w-0">
              <div className="text-[10px] font-semibold uppercase tracking-wider text-purple-300 flex items-center gap-1">
                <Sparkles size={10} />
                <span>
                  Interactive Tour • {stepIndex + 1} of {WALKTHROUGH_STEPS.length}
                </span>
              </div>
              <h3 className="text-sm font-semibold text-white truncate">
                {currentStep.title}
              </h3>
            </div>
          </div>
          <button
            type="button"
            onClick={onDismiss}
            aria-label="Dismiss welcome guide"
            title="Skip walkthrough"
            className="p-1 rounded-md text-gray-400 hover:text-white hover:bg-white/10 transition-colors flex-shrink-0"
          >
            <X size={14} />
          </button>
        </div>

        {/* Body */}
        <p className="text-gray-300 leading-relaxed mb-2.5">
          {currentStep.description}
        </p>

        {/* Tip Box */}
        <div className="px-2.5 py-1.5 rounded-md bg-[#10131a] border border-[#262b38] text-[11px] text-gray-400 mb-3">
          <span className="text-purple-300 font-medium">Tip: </span>
          {currentStep.tip}
        </div>

        {/* Footer Controls */}
        <div className="flex items-center justify-between gap-2 pt-2 border-t border-[#262b38]">
          {/* Step Dots */}
          <div className="flex items-center gap-1.5">
            {WALKTHROUGH_STEPS.map((s, idx) => (
              <button
                key={s.id}
                type="button"
                onClick={() => setStepIndex(idx)}
                aria-label={`Go to step ${idx + 1}: ${s.title}`}
                className={`h-1.5 rounded-full transition-all ${
                  idx === stepIndex
                    ? "w-4 bg-purple-400"
                    : "w-1.5 bg-gray-600 hover:bg-gray-400"
                }`}
              />
            ))}
          </div>

          <div className="flex items-center gap-1.5">
            <button
              type="button"
              onClick={onDismiss}
              className="px-2 py-1 rounded text-gray-400 hover:text-gray-200 transition-colors"
            >
              Skip
            </button>
            {stepIndex > 0 && (
              <button
                type="button"
                onClick={() => setStepIndex((i) => Math.max(0, i - 1))}
                className="inline-flex items-center gap-0.5 px-2 py-1 rounded bg-[#232836] hover:bg-[#2d3344] text-gray-200 font-medium transition-colors"
              >
                <ChevronLeft size={13} />
                Back
              </button>
            )}
            {!isLastStep ? (
              <button
                type="button"
                onClick={() =>
                  setStepIndex((i) => Math.min(WALKTHROUGH_STEPS.length - 1, i + 1))
                }
                className="inline-flex items-center gap-0.5 px-2.5 py-1 rounded bg-purple-600 hover:bg-purple-500 text-white font-medium transition-colors"
              >
                Next
                <ChevronRight size={13} />
              </button>
            ) : (
              <button
                type="button"
                onClick={() => {
                  onDismiss();
                  onOpenGuideDrawer?.();
                }}
                className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-medium transition-colors"
              >
                Finish
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default ShellWalkthrough;
