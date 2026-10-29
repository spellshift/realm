import React, { useMemo, useState } from "react";
import {
  BookOpen,
  Code2,
  Compass,
  ExternalLink,
  Keyboard,
  Layers,
  Play,
  Search,
  ShieldAlert,
  Terminal,
  X,
  Zap,
} from "lucide-react";
import docsData from "../../../assets/eldritch-docs.json";

const docsMap = docsData as Record<string, { signature: string; description: string }>;

interface ShellNuxDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  portalId: number | null;
  onRunCode: (code: string) => void;
  onStageToPrompt?: (code: string) => void;
  onLoadIntoComposer: (code: string) => void;
  onSpawnFreshShell: () => void;
  onStartWalkthrough?: () => void;
}

const ShellNuxDrawer: React.FC<ShellNuxDrawerProps> = ({
  isOpen,
  onClose,
  portalId,
  onRunCode,
  onLoadIntoComposer,
  onSpawnFreshShell,
  onStartWalkthrough,
}) => {
  const [activeTab, setActiveTab] = useState<"guide" | "docs">("guide");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedModule, setSelectedModule] = useState<string>("all");

  const modules = useMemo(() => {
    const modSet = new Set<string>();
    Object.keys(docsMap).forEach((key) => {
      if (key.includes(".")) {
        modSet.add(key.split(".")[0]);
      }
    });
    return Array.from(modSet).sort();
  }, []);

  const filteredDocs = useMemo(() => {
    return Object.entries(docsMap)
      .filter(([key, val]) => {
        if (!key.includes(".")) return false;
        const mod = key.split(".")[0];
        if (selectedModule !== "all" && mod !== selectedModule) return false;
        if (searchQuery.trim()) {
          const q = searchQuery.toLowerCase();
          return (
            key.toLowerCase().includes(q) ||
            val.signature.toLowerCase().includes(q) ||
            val.description.toLowerCase().includes(q)
          );
        }
        return true;
      })
      .slice(0, 60);
  }, [selectedModule, searchQuery]);

  if (!isOpen) return null;

  return (
    <aside
      aria-label="Operator Guide & Eldritch Reference"
      data-testid="shell-nux-drawer"
      className="w-full md:w-96 lg:w-[440px] flex-shrink-0 flex flex-col rounded-md bg-[#161920] border border-[#2e3440] h-full overflow-hidden shadow-xl"
    >
      {/* Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 bg-[#1b1f2a] border-b border-[#2e3440]">
        <div className="flex items-center gap-2">
          <Compass size={16} className="text-purple-400" />
          <h2 className="text-sm font-semibold text-white">Operator Guide &amp; Eldritch Reference</h2>
        </div>
        <button
          type="button"
          onClick={onClose}
          aria-label="Close operator guide"
          className="p-1 rounded text-gray-400 hover:text-white hover:bg-[#2a2f3d] transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      {/* Sub-tabs */}
      <div className="flex border-b border-[#2e3440] bg-[#181b24] text-xs">
        <button
          type="button"
          onClick={() => setActiveTab("guide")}
          className={`flex-1 py-2 px-3 font-medium border-b-2 transition-colors ${
            activeTab === "guide"
              ? "border-purple-500 text-white bg-[#1d222e]"
              : "border-transparent text-gray-400 hover:text-gray-200"
          }`}
        >
          Quick Start Guide
        </button>
        <button
          type="button"
          onClick={() => setActiveTab("docs")}
          className={`flex-1 py-2 px-3 font-medium border-b-2 transition-colors ${
            activeTab === "docs"
              ? "border-purple-500 text-white bg-[#1d222e]"
              : "border-transparent text-gray-400 hover:text-gray-200"
          }`}
        >
          Eldritch v0.4.1 Stdlib ({modules.length} Modules)
        </button>
      </div>

      {activeTab === "guide" ? (
        <div className="flex-1 overflow-y-auto shell-scrollbar p-3.5 space-y-3.5 text-xs text-gray-300">
          {onStartWalkthrough && (
            <button
              type="button"
              onClick={onStartWalkthrough}
              className="w-full flex items-center justify-between px-3 py-2 rounded-md bg-purple-600/20 hover:bg-purple-600/30 border border-purple-500/40 text-purple-200 font-medium transition-colors"
            >
              <span className="flex items-center gap-2">
                <Compass size={14} className="text-purple-400" />
                Replay Interactive Walkthrough
              </span>
              <span className="text-[11px] text-purple-300">Start Tour →</span>
            </button>
          )}
          {/* Card 1: Queued vs Interactive Mode */}
          <div className="rounded-md bg-[#1c202b] border border-[#2d3446] p-3 space-y-2">
            <div className="flex items-center gap-2 text-white font-semibold">
              <Zap size={15} className="text-emerald-400" />
              <span>1. Queued Mode vs. Interactive Portal Mode</span>
            </div>
            <p className="text-gray-400 leading-relaxed">
              By default, shells start in <strong className="text-gray-200">Queued Mode</strong>: commands you enter are queued and executed in sequence on the beacon&apos;s next callback check-in.
            </p>
            <p className="text-gray-400 leading-relaxed">
              To switch to real-time <strong className="text-emerald-300">Interactive Mode</strong> (sub-second output + PTY &amp; SSH pivoting), open a Portal stream:
            </p>
            <div className="flex items-center justify-between bg-[#12151c] px-2.5 py-1.5 rounded border border-[#282e3d]">
              <code className="font-mono text-emerald-300">portal.create()</code>
              <button
                type="button"
                disabled={!!portalId}
                onClick={() => onRunCode("portal.create()")}
                className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-600/80 hover:bg-emerald-600 disabled:opacity-40 text-white font-medium transition-colors"
              >
                <Play size={11} />
                {portalId ? `Portal #${portalId} Active` : "Run portal.create()"}
              </button>
            </div>
          </div>

          {/* Card 2: Macros & Pivoting */}
          <div className="rounded-md bg-[#1c202b] border border-[#2d3446] p-3 space-y-2">
            <div className="flex items-center gap-2 text-white font-semibold">
              <Layers size={15} className="text-purple-400" />
              <span>2. Shell Macros &amp; Lateral Pivots (PTY / SSH)</span>
            </div>
            <ul className="space-y-1.5 text-gray-400">
              <li>
                • <code className="font-mono text-purple-300">!whoami</code> — Shorthand macro for{" "}
                <code className="font-mono text-gray-200">sys.shell(&quot;whoami&quot;)</code>
              </li>
              <li>
                • <code className="font-mono text-purple-300">pty()</code> — Opens a full interactive OS terminal tab over an active portal
              </li>
              <li>
                • <code className="font-mono text-purple-300">ssh(&quot;user:pass@10.0.0.5:22&quot;)</code> — Negotiates an SSH session through the agent&apos;s portal in a dedicated tab
              </li>
              <li>
                • <code className="font-mono text-purple-300">help(file)</code> — Prints inline documentation for any Eldritch module
              </li>
            </ul>
          </div>

          {/* Card 3: Slow Beacons vs Stuck Interpreters */}
          <div className="rounded-md bg-[#1c202b] border border-[#2d3446] p-3 space-y-2">
            <div className="flex items-center gap-2 text-white font-semibold">
              <ShieldAlert size={15} className="text-amber-400" />
              <span>3. Long Callback Intervals &amp; Stuck Interpreters</span>
            </div>
            <p className="text-gray-400 leading-relaxed">
              <strong className="text-gray-200">Slow Beacons (&gt;10m callback):</strong> You can queue multiple commands back-to-back at any time. They will wait in the queue and execute in exact order when the beacon checks in.
            </p>
            <p className="text-gray-400 leading-relaxed">
              <strong className="text-amber-200">Stuck Interpreter:</strong> Each shell connects to a persistent Eldritch interpreter on the agent. If a blocking command hangs that interpreter after check-in, use{" "}
              <strong className="text-white">Spawn new shell</strong> to immediately start a clean interpreter without losing this shell&apos;s history.
            </p>
            <button
              type="button"
              onClick={onSpawnFreshShell}
              className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-amber-500/20 hover:bg-amber-500/30 border border-amber-500/40 text-amber-200 font-medium transition-colors"
            >
              <Terminal size={12} />
              Spawn new shell
            </button>
          </div>

          {/* Card 4: Keyboard Shortcuts */}
          <div className="rounded-md bg-[#1c202b] border border-[#2d3446] p-3 space-y-2">
            <div className="flex items-center gap-2 text-white font-semibold">
              <Keyboard size={15} className="text-blue-400" />
              <span>4. Keyboard Shortcuts</span>
            </div>
            <div className="grid grid-cols-2 gap-1.5 text-[11px]">
              <div className="flex justify-between bg-[#14171f] px-2 py-1 rounded">
                <span className="text-gray-400">Autocomplete</span>
                <kbd className="font-mono text-gray-200">Tab / Ctrl+Space</kbd>
              </div>
              <div className="flex justify-between bg-[#14171f] px-2 py-1 rounded">
                <span className="text-gray-400">History Search</span>
                <kbd className="font-mono text-gray-200">Ctrl+R</kbd>
              </div>
              <div className="flex justify-between bg-[#14171f] px-2 py-1 rounded">
                <span className="text-gray-400">Cancel Input</span>
                <kbd className="font-mono text-gray-200">Ctrl+C</kbd>
              </div>
              <div className="flex justify-between bg-[#14171f] px-2 py-1 rounded">
                <span className="text-gray-400">Clear Screen</span>
                <kbd className="font-mono text-gray-200">Ctrl+L</kbd>
              </div>
              <div className="flex justify-between bg-[#14171f] px-2 py-1 rounded">
                <span className="text-gray-400">Switch Tabs</span>
                <kbd className="font-mono text-gray-200">Ctrl+1..9</kbd>
              </div>
              <div className="flex justify-between bg-[#14171f] px-2 py-1 rounded">
                <span className="text-gray-400">Run Composer</span>
                <kbd className="font-mono text-gray-200">Ctrl+Enter</kbd>
              </div>
            </div>
          </div>

          {/* External Documentation Link */}
          <a
            href="https://docs.realm.pub/user-guide/eldritch"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-between px-3 py-2 rounded-md bg-purple-950/40 hover:bg-purple-900/50 border border-purple-500/30 text-purple-200 font-medium transition-colors"
          >
            <span className="flex items-center gap-2">
              <BookOpen size={14} />
              Read Full Eldritch Documentation
            </span>
            <ExternalLink size={13} />
          </a>
        </div>
      ) : (
        /* Eldritch Standard Library Explorer Tab */
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="p-2.5 border-b border-[#2e3440] space-y-2">
            <div className="relative">
              <Search size={14} className="text-gray-500 absolute left-2.5 top-2.5" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search Eldritch functions (e.g. file.list, process, ssh)..."
                className="w-full pl-8 pr-3 py-1.5 rounded bg-[#101217] border border-[#2e3440] text-xs text-gray-200 placeholder-gray-500 focus:outline-none focus:border-purple-500"
              />
            </div>

            <div className="flex items-center gap-1 overflow-x-auto pb-1 text-[11px]">
              <button
                type="button"
                onClick={() => setSelectedModule("all")}
                className={`px-2 py-0.5 rounded flex-shrink-0 font-medium ${
                  selectedModule === "all"
                    ? "bg-purple-600 text-white"
                    : "bg-[#1e222d] text-gray-400 hover:text-gray-200"
                }`}
              >
                All
              </button>
              {modules.map((mod) => (
                <button
                  key={mod}
                  type="button"
                  onClick={() => setSelectedModule(mod)}
                  className={`px-2 py-0.5 rounded flex-shrink-0 font-mono ${
                    selectedModule === mod
                      ? "bg-purple-600 text-white"
                      : "bg-[#1e222d] text-gray-400 hover:text-gray-200"
                  }`}
                >
                  {mod}
                </button>
              ))}
            </div>
          </div>

          <div className="flex-1 overflow-y-auto shell-scrollbar p-2.5 space-y-2">
            {filteredDocs.map(([fnName, info]) => {
              const callTemplate = `${fnName}()`;
              return (
                <div
                  key={fnName}
                  className="rounded-md bg-[#1b1f29] border border-[#2b3140] p-2.5 text-xs space-y-1.5"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-mono font-semibold text-purple-300">{fnName}</span>
                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        onClick={() => onLoadIntoComposer(callTemplate)}
                        title="Open in Composer"
                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-200 text-[11px]"
                      >
                        <Code2 size={10} />
                        Composer
                      </button>
                    </div>
                  </div>
                  <div className="font-mono text-[11px] text-cyan-300 bg-[#12151c] px-2 py-1 rounded overflow-x-auto">
                    {info.signature}
                  </div>
                  <p className="text-gray-400 text-[11px] line-clamp-3">
                    {info.description.split("\n")[0]}
                  </p>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </aside>
  );
};

export default ShellNuxDrawer;
