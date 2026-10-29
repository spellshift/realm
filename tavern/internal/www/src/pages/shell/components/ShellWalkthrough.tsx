import React, { useEffect, useState } from "react";
import {
  BookMarked,
  Bug,
  ChevronLeft,
  ChevronRight,
  Clock,
  Code2,
  Compass,
  Database,
  History,
  Loader2,
  Network,
  Power,
  Shell,
  Sparkles,
  Terminal,
  X,
} from "lucide-react";

export interface WalkthroughStep {
  id: string;
  selector: string;
  fallbackSelector?: string;
  title: string;
  description: string;
  tip?: string;
  icon: React.ReactNode;
}

export const WALKTHROUGH_STEPS: WalkthroughStep[] = [
  {
    id: "terminal",
    selector: '[data-tour="shell-terminal"]',
    title: "Welcome to the Eldritch REPL",
    description:
      "Write Eldritch directly into the terminal for it to be executed on the agent.",
    tip: "You can use the ! macro to quickly run shell commands, such as !ls",
    icon: <Terminal size={16} className="text-purple-400" />,
  },
  {
    id: "composer",
    selector: '[data-tour="shell-composer"]',
    fallbackSelector: '[data-tour="shell-composer-toggle"]',
    title: "Eldritch Composer",
    description:
      "For more complex scripts, it may be easier to write them in the Composer view.",
    icon: <Code2 size={16} className="text-purple-400" />,
  },
  {
    id: "composer-tomes",
    selector: '[data-tour="shell-composer-tomes"]',
    fallbackSelector: '[data-tour="shell-composer"]',
    title: "Load Tomes",
    description:
      "Search and load pre-built Eldritch tomes directly into the script view to save time.",
    icon: <BookMarked size={16} className="text-purple-400" />,
  },
  {
    id: "composer-rc",
    selector: '[data-tour="shell-composer-rc"]',
    fallbackSelector: '[data-tour="shell-composer"]',
    title: "RC Script",
    description:
      "Edit your shared RC script (pre-loaded with shell aliases) and toggle it on to automatically run the first time you connect to a shell.",
    icon: <Power size={16} className="text-purple-400" />,
  },
  {
    id: "non-interactive",
    selector: '[data-tour="shell-non-interactive"]',
    title: "Async Mode",
    description:
      "All code runs on the agent, but in async mode we queue it for the agent to retrieve during its next callback. This means you'll need to wait for agent callbacks before you'll see any output from your Eldritch, but it's typically more stealthy to operate in this mode.",
    icon: <Clock size={16} className="text-yellow-400" />,
  },
  {
    id: "callback",
    selector: '[data-tour="shell-next-callback"]',
    fallbackSelector: '[data-tour="shell-non-interactive"]',
    title: "Agent Callback",
    description:
      "This timer shows when the next agent callback is expected and will alert you if an agent misses its check-in window.",
    icon: <Clock size={16} className="text-blue-400" />,
  },
  {
    id: "queue-status",
    selector: '[data-tour="shell-queue-indicator"]',
    fallbackSelector: '[data-tour="shell-status-bar"]',
    title: "Queued & Executing Tasks",
    description:
      "When tasks are queued or executing on the agent, hover or click this indicator to see what's currently in the queue.",
    icon: <Loader2 size={16} className="text-amber-400" />,
  },
  {
    id: "knowledge",
    selector: '[data-tour="shell-knowledge"]',
    title: "Knowledge Graph",
    description:
      "View credentials, hosts, and files reported across your engagement, and quickly use them to stage commands or launch pivots.",
    tip: "Items reported via the Eldritch report module appear here automatically.",
    icon: <Database size={16} className="text-emerald-400" />,
  },
  {
    id: "pivots",
    selector: '[data-tour="shell-pivots"]',
    title: "Pivots",
    description:
      "Launch interactive PTY or SSH sessions through an active portal and review saved transcripts from past pivots.",
    icon: <Network size={16} className="text-purple-400" />,
  },
  {
    id: "history",
    selector: '[data-tour="shell-history"]',
    title: "Command Queue & History",
    description:
      "Inspect queued, running, and completed commands. View full output, re-run previous commands, or load them into the Composer.",
    tip: "Filter by Queued / Running to see tasks waiting for the next beacon callback.",
    icon: <History size={16} className="text-blue-400" />,
  },
  {
    id: "guide",
    selector: '[data-tour="shell-guide"]',
    title: "Operator Guide & Stdlib Reference",
    description:
      "Search the Eldritch standard library documentation, review keyboard shortcuts, or replay this walkthrough anytime.",
    tip: "Load any standard library function directly into the Composer from the reference tab.",
    icon: <Compass size={16} className="text-amber-400" />,
  },
  {
    id: "portal",
    selector: '[data-tour="shell-new-portal"]',
    fallbackSelector: '[data-tour="shell-actions-button"]',
    title: "Going Interactive",
    description:
      'For real-time interaction with the agent, you can open a "Portal" on it. This creates an established connection which can be used for low-latency operations and to establish pivots (e.g. SSH).',
    tip: "You can always close a portal when you're finished and open one later. But be warned, doing so will terminate your active pivots.",
    icon: <Shell size={16} className="text-emerald-400" />,
  },
  {
    id: "feedback",
    selector: '[data-tour="shell-bug-report"]',
    fallbackSelector: '[data-tour="shell-actions-button"]',
    title: "Feedback & Bug Reports",
    description:
      "Run into an issue or have an idea to improve the shell? Click here anytime to report a bug or share your feedback. Thank you for helping make Realm better!",
    icon: <Bug size={16} className="text-pink-400" />,
  },
];

interface ShellWalkthroughProps {
  isOpen: boolean;
  onDismiss: () => void;
  onOpenGuideDrawer?: () => void;
  onOpenComposer?: () => void;
  onPortalStepChange?: (isActive: boolean) => void;
  onQueueStepChange?: (isActive: boolean) => void;
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
  onOpenComposer,
  onPortalStepChange,
  onQueueStepChange,
}) => {
  const [stepIndex, setStepIndex] = useState(0);
  const [targetRect, setTargetRect] = useState<TargetRect | null>(null);

  const currentStep = WALKTHROUGH_STEPS[stepIndex] || WALKTHROUGH_STEPS[0];

  useEffect(() => {
    if (!isOpen) {
      setStepIndex(0);
      onPortalStepChange?.(false);
      onQueueStepChange?.(false);
      return;
    }

    if (
      currentStep.id === "composer" ||
      currentStep.id === "composer-tomes" ||
      currentStep.id === "composer-rc"
    ) {
      if (onOpenComposer) {
        onOpenComposer();
      } else if (!document.querySelector('[data-tour="shell-composer"]')) {
        const toggleBtn = document.querySelector<HTMLElement>(
          '[data-tour="shell-composer-toggle"]'
        );
        toggleBtn?.click();
      }
    }

    onPortalStepChange?.(currentStep.id === "portal" || currentStep.id === "feedback");
    onQueueStepChange?.(currentStep.id === "queue-status");

    const updateRect = () => {
      const el =
        document.querySelector(currentStep.selector) ||
        (currentStep.fallbackSelector
          ? document.querySelector(currentStep.fallbackSelector)
          : null);
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
    const timer = setTimeout(updateRect, 30);
    window.addEventListener("resize", updateRect);
    window.addEventListener("scroll", updateRect, true);
    return () => {
      clearTimeout(timer);
      window.removeEventListener("resize", updateRect);
      window.removeEventListener("scroll", updateRect, true);
    };
  }, [
    isOpen,
    stepIndex,
    currentStep.id,
    currentStep.selector,
    currentStep.fallbackSelector,
    onOpenComposer,
    onPortalStepChange,
    onQueueStepChange,
  ]);

  useEffect(() => {
    return () => {
      onPortalStepChange?.(false);
      onQueueStepChange?.(false);
    };
  }, [onPortalStepChange, onQueueStepChange]);

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
    const popoverEstH = currentStep.tip ? 260 : 230;

    // If highlighting the large terminal area, dock the popover cleanly in the top-right of the terminal
    if (currentStep.id === "terminal") {
      return {
        top: `${Math.max(76, targetRect.top + 16)}px`,
        right: "28px",
      };
    }

    // If highlighting "New Portal" inside the right-aligned actions menu, position popover to the left of the menu
    if (currentStep.id === "portal" && targetRect.left > popoverW + 32) {
      return {
        top: `${Math.max(56, Math.min(targetRect.top - 12, viewportH - popoverEstH - 16))}px`,
        left: `${Math.max(16, targetRect.left - popoverW - 20)}px`,
      };
    }

    const belowTop = targetRect.top + targetRect.height + 14;
    const clampedLeft = Math.max(
      16,
      Math.min(
        targetRect.left + targetRect.width / 2 - popoverW / 2,
        viewportW - popoverW - 16
      )
    );

    if (belowTop + popoverEstH < viewportH) {
      return {
        top: `${belowTop}px`,
        left: `${clampedLeft}px`,
      };
    }

    return {
      bottom: `${Math.max(16, viewportH - targetRect.top + 14)}px`,
      left: `${clampedLeft}px`,
    };
  };

  return (
    <div
      data-testid="shell-nux-banner"
      aria-label="Shell interactive walkthrough"
      className="fixed inset-0 z-50 pointer-events-none shell-font-onest"
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
        {currentStep.tip && (
          <div className="px-2.5 py-1.5 rounded-md bg-[#10131a] border border-[#262b38] text-[11px] text-gray-400 mb-3">
            <span className="text-purple-300 font-medium">Tip: </span>
            {currentStep.tip}
          </div>
        )}

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
