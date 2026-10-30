import React, { useEffect, useRef, useState } from "react";
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
  NotebookPen,
  Settings,
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
    id: "session-settings",
    selector: '[data-tour="shell-session-settings-toggle"]',
    title: "RC Script",
    description:
      "Click the cogwheel on the bottom bar to open Session Settings and configure an RC script that runs automatically when you connect to a shell.",
    icon: <Settings size={16} className="text-gray-300" />,
  },
  {
    id: "setup-aliases",
    selector: '[data-tour="shell-setup-aliases"]',
    fallbackSelector: '[data-tour="shell-session-settings"]',
    title: "Set Default Aliases",
    description:
      "Click Set Default Aliases to enable common shell helper functions (like ls, ps, cat, and whoami) across your shells in one click, or load another RC tome or custom script.",
    icon: <BookMarked size={16} className="text-purple-400" />,
  },
  {
    id: "composer",
    selector: '[data-tour="shell-composer-toggle"]',
    fallbackSelector: '[data-tour="shell-composer"]',
    title: "Eldritch Composer",
    description:
      "For more complex scripts, it may be easier to write them in the Composer view.",
    icon: <Code2 size={16} className="text-purple-400" />,
  },
  {
    id: "scratchpad",
    selector: '[data-tour="shell-scratchpad-toggle"]',
    fallbackSelector: '[data-tour="shell-scratchpad"]',
    title: "Scratchpad",
    description:
      "A place for you to store notes, which persists across sessions for easy access.",
    icon: <NotebookPen size={16} className="text-purple-400" />,
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
    id: "non-interactive",
    selector: '[data-tour="shell-non-interactive"]',
    title: "Async Mode",
    description:
      "All code runs on the agent, but in async mode we queue it for the agent to retrieve during its next callback. Click this status indicator anytime to open or close an interactive portal.",
    icon: <Clock size={16} className="text-yellow-400" />,
  },
  {
    id: "context",
    selector: '[data-tour="shell-knowledge"]',
    title: "Context",
    description:
      "Explore Realm's knowledge graph with a view tailored for your operation. Your pivoting sessions, PTYs, and other operational info are stored here.",
    tip: "Items reported via the Eldritch report module and successful pivots appear here automatically.",
    icon: <Database size={16} className="text-emerald-400" />,
  },
  {
    id: "portal",
    selector: '[data-tour="shell-new-portal"]',
    fallbackSelector: '[data-tour="shell-non-interactive"]',
    title: "Going Interactive",
    description:
      'For real-time interaction with the agent, you can open a "Portal" from the Context drawer or the bottom status bar. This creates an established connection for low-latency operations and interactive PTY or SSH pivots.',
    tip: "You can always close a portal when you're finished and open one later. Doing so will terminate your active pivots.",
    icon: <Shell size={16} className="text-emerald-400" />,
  },
  {
    id: "history",
    selector: '[data-tour="shell-history"]',
    title: "History",
    description:
      "Inspect queued, running, and completed commands across all of your shell sessions, not just the active one. View full output, re-run previous commands, or load them into the Composer.",
    icon: <History size={16} className="text-blue-400" />,
  },
  {
    id: "guide",
    selector: '[data-tour="shell-guide"]',
    fallbackSelector: '[data-tour="shell-actions-button"]',
    title: "Eldritch Documentation",
    description:
      "Open the Eldritch user guide and standard library reference, or replay this tutorial anytime from the actions menu.",
    icon: <Compass size={16} className="text-amber-400" />,
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
  onCloseComposer?: () => void;
  onOpenScratchpad?: () => void;
  onCloseScratchpad?: () => void;
  onOpenSessionSettings?: () => void;
  onCloseSessionSettings?: () => void;
  onOpenContextDrawer?: () => void;
  onCloseContextDrawer?: () => void;
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
  onCloseComposer,
  onOpenScratchpad,
  onCloseScratchpad,
  onOpenSessionSettings,
  onCloseSessionSettings,
  onOpenContextDrawer,
  onCloseContextDrawer,
  onPortalStepChange,
  onQueueStepChange,
}) => {
  const [stepIndex, setStepIndex] = useState(0);
  const [targetRect, setTargetRect] = useState<TargetRect | null>(null);

  const callbacksRef = useRef({
    onOpenComposer,
    onCloseComposer,
    onOpenScratchpad,
    onCloseScratchpad,
    onOpenSessionSettings,
    onCloseSessionSettings,
    onOpenContextDrawer,
    onCloseContextDrawer,
    onPortalStepChange,
    onQueueStepChange,
  });
  callbacksRef.current = {
    onOpenComposer,
    onCloseComposer,
    onOpenScratchpad,
    onCloseScratchpad,
    onOpenSessionSettings,
    onCloseSessionSettings,
    onOpenContextDrawer,
    onCloseContextDrawer,
    onPortalStepChange,
    onQueueStepChange,
  };

  const currentStep = WALKTHROUGH_STEPS[stepIndex] || WALKTHROUGH_STEPS[0];

  useEffect(() => {
    const cb = callbacksRef.current;
    if (!isOpen) {
      setStepIndex(0);
      cb.onPortalStepChange?.(false);
      cb.onQueueStepChange?.(false);
      return;
    }

    if (currentStep.id === "composer") {
      if (cb.onOpenComposer) {
        cb.onOpenComposer();
      } else if (!document.querySelector('[data-tour="shell-composer"]')) {
        const toggleBtn = document.querySelector<HTMLElement>(
          '[data-tour="shell-composer-toggle"]'
        );
        toggleBtn?.click();
      }
    } else {
      cb.onCloseComposer?.();
    }

    if (currentStep.id === "scratchpad") {
      if (cb.onOpenScratchpad) {
        cb.onOpenScratchpad();
      } else if (!document.querySelector('[data-tour="shell-scratchpad"]')) {
        const toggleBtn = document.querySelector<HTMLElement>(
          '[data-tour="shell-scratchpad-toggle"]'
        );
        toggleBtn?.click();
      }
    } else {
      cb.onCloseScratchpad?.();
    }

    if (currentStep.id === "setup-aliases") {
      if (cb.onOpenSessionSettings) {
        cb.onOpenSessionSettings();
      } else if (!document.querySelector('[data-tour="shell-session-settings"]')) {
        const toggleBtn = document.querySelector<HTMLElement>(
          '[data-tour="shell-session-settings-toggle"]'
        );
        toggleBtn?.click();
      }
    } else {
      cb.onCloseSessionSettings?.();
    }

    if (currentStep.id === "portal") {
      cb.onOpenContextDrawer?.();
    } else if (currentStep.id === "context" || currentStep.id === "history") {
      cb.onCloseContextDrawer?.();
    }

    const handleTargetClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      if (!target) return;
      if (
        currentStep.id === "session-settings" &&
        target.closest('[data-tour="shell-session-settings-toggle"]')
      ) {
        setStepIndex((i) => Math.min(WALKTHROUGH_STEPS.length - 1, i + 1));
      } else if (
        currentStep.id === "setup-aliases" &&
        target.closest('[data-tour="shell-setup-aliases"]')
      ) {
        setStepIndex((i) => Math.min(WALKTHROUGH_STEPS.length - 1, i + 1));
      }
    };

    document.addEventListener("click", handleTargetClick, true);

    cb.onPortalStepChange?.(currentStep.id === "guide" || currentStep.id === "feedback");
    cb.onQueueStepChange?.(currentStep.id === "queue-status");

    const updateRect = (allowClear = false, allowFallback = true) => {
      const el =
        document.querySelector(currentStep.selector) ||
        (allowFallback && currentStep.fallbackSelector
          ? document.querySelector(currentStep.fallbackSelector)
          : null);
      if (!el) {
        if (allowClear) {
          setTargetRect(null);
        }
        return false;
      }
      const r = el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) {
        if (allowClear) {
          setTargetRect(null);
        }
        return false;
      }
      setTargetRect({
        top: r.top,
        left: r.left,
        width: r.width,
        height: r.height,
      });
      return true;
    };

    updateRect(false, false);
    const rafId = window.requestAnimationFrame(() => {
      updateRect(false, true);
    });
    const timer1 = setTimeout(() => updateRect(true, true), 50);
    const timer2 = setTimeout(() => updateRect(true, true), 120);
    const timer3 = setTimeout(() => updateRect(true, true), 250);
    const handleViewportChange = () => updateRect(false, true);
    window.addEventListener("resize", handleViewportChange);
    window.addEventListener("scroll", handleViewportChange, true);
    return () => {
      window.cancelAnimationFrame(rafId);
      clearTimeout(timer1);
      clearTimeout(timer2);
      clearTimeout(timer3);
      document.removeEventListener("click", handleTargetClick, true);
      window.removeEventListener("resize", handleViewportChange);
      window.removeEventListener("scroll", handleViewportChange, true);
    };
  }, [
    isOpen,
    stepIndex,
    currentStep.id,
    currentStep.selector,
    currentStep.fallbackSelector,
  ]);

  useEffect(() => {
    return () => {
      callbacksRef.current.onPortalStepChange?.(false);
      callbacksRef.current.onQueueStepChange?.(false);
    };
  }, []);

  if (!isOpen) return null;

  const isLastStep = stepIndex === WALKTHROUGH_STEPS.length - 1;

  const handleDismiss = () => {
    if (currentStep.id === "portal") {
      onCloseContextDrawer?.();
    }
    if (currentStep.id === "composer") {
      onCloseComposer?.();
    }
    if (currentStep.id === "scratchpad") {
      onCloseScratchpad?.();
    }
    if (currentStep.id === "setup-aliases") {
      onCloseSessionSettings?.();
    }
    onDismiss();
  };

  // Compute popover position near the highlighted element
  const getPopoverStyle = (): React.CSSProperties => {
    const viewportW = typeof window !== "undefined" ? window.innerWidth : 1280;
    const viewportH = typeof window !== "undefined" ? window.innerHeight : 800;
    const popoverW = 384;
    const popoverEstH = currentStep.tip ? 260 : 230;

    if (!targetRect) {
      return {
        top: "88px",
        left: `${Math.max(16, viewportW - popoverW - 24)}px`,
      };
    }

    // If highlighting the large terminal area, dock the popover cleanly in the top-right of the terminal
    if (currentStep.id === "terminal") {
      return {
        top: `${Math.max(76, targetRect.top + 16)}px`,
        left: `${Math.max(16, viewportW - popoverW - 28)}px`,
      };
    }

    // If highlighting an element inside the right-aligned actions menu or Context drawer, position popover to the left
    if (
      (currentStep.id === "portal" ||
        currentStep.id === "guide" ||
        currentStep.id === "feedback") &&
      targetRect.left > popoverW + 32
    ) {
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
          className="fixed rounded-lg border-2 border-purple-400/90 transition-all duration-300 ease-out pointer-events-none"
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
        <div className="fixed inset-0 bg-black/50 transition-opacity duration-300 ease-out pointer-events-none" />
      )}

      {/* Floating Walkthrough Popover */}
      <div
        role="dialog"
        aria-modal="false"
        aria-label={currentStep.title}
        style={getPopoverStyle()}
        className="fixed w-96 max-w-[calc(100vw-32px)] rounded-xl bg-gray-900 border border-purple-500/40 shadow-2xl p-4 text-xs text-gray-200 pointer-events-auto transition-all duration-300 ease-out"
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
            onClick={handleDismiss}
            aria-label="Dismiss welcome guide"
            title="Skip walkthrough"
            className="p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors flex-shrink-0"
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
          <div className="px-2.5 py-1.5 rounded-md bg-gray-800/70 border border-gray-700 text-[11px] text-gray-400 mb-3">
            <span className="text-purple-300 font-medium">Tip: </span>
            {currentStep.tip}
          </div>
        )}

        {/* Footer Controls */}
        <div className="flex items-center justify-between gap-2 pt-2 border-t border-gray-800">
          {/* Step Dots */}
          <div className="flex items-center gap-1 min-w-0">
            {WALKTHROUGH_STEPS.map((s, idx) => (
              <button
                key={s.id}
                type="button"
                onClick={() => setStepIndex(idx)}
                aria-label={`Go to step ${idx + 1}: ${s.title}`}
                className={`h-1.5 rounded-full transition-all shrink-0 ${
                  idx === stepIndex
                    ? "w-3 bg-purple-400"
                    : "w-1 bg-gray-600 hover:bg-gray-400"
                }`}
              />
            ))}
          </div>

          <div className="flex items-center gap-1.5 shrink-0">
            <button
              type="button"
              onClick={handleDismiss}
              className="px-2 py-1 rounded text-gray-400 hover:text-gray-200 transition-colors"
            >
              Skip
            </button>
            {stepIndex > 0 && (
              <button
                type="button"
                onClick={() => setStepIndex((i) => Math.max(0, i - 1))}
                className="inline-flex items-center gap-0.5 px-2 py-1 rounded bg-gray-800 hover:bg-gray-700 text-gray-200 font-medium transition-colors"
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
                  handleDismiss();
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
