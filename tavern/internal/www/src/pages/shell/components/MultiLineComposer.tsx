import React, { useState, useRef, useEffect } from "react";
import { Code2, Play, CornerDownLeft, Trash2, ChevronDown, ChevronUp, Sparkles } from "lucide-react";

export interface EldritchSnippet {
  label: string;
  description: string;
  code: string;
}

export const ELDRITCH_SNIPPETS: EldritchSnippet[] = [
  {
    label: "System & Identity Recon",
    description: "Gather host OS info, current user, hostname, and network interfaces",
    code: `info = sys.info()
u = user.info()
print(f"Host: {sys.hostname()} ({info['os']} {info['arch']})")
print(f"User: {u}")
for iface in net.interfaces():
    print(iface)`,
  },
  {
    label: "Upgrade to Interactive Portal",
    description: "Open a low-latency portal stream for real-time REPL, PTY, and SSH pivots",
    code: `portal.create()`,
  },
  {
    label: "Process & Listening Ports Audit",
    description: "Enumerate running processes and active network connections",
    code: `for p in process.list():
    print(f"[{p['pid']}] {p['username']} - {p['name']} {p['command']}")
for c in net.conns():
    print(c)`,
  },
  {
    label: "Directory Listing & SHA-256 Hashes",
    description: "List files in a directory and compute SHA-256 hashes for regular files",
    code: `target_dir = "/tmp"
for entry in file.list(target_dir):
    full_path = f"{target_dir}/{entry['file_name']}"
    if entry["type"] == "File":
        print(f"{entry['file_name']} ({entry['size']}B) sha256={hash.file(full_path)}")
    else:
        print(f"[DIR] {entry['file_name']}")`,
  },
  {
    label: "Fast Callback Interval (5s)",
    description: "Reduce beacon callback interval to 5 seconds for rapid tasking",
    code: `agent.set_callback_interval(5)
print(f"Updated callback interval to {agent.callback_interval()}s")`,
  },
];

interface MultiLineComposerProps {
  isOpen: boolean;
  onToggle: () => void;
  code: string;
  onChange: (value: string) => void;
  onRunCode: (code: string) => void;
  onStageToPrompt: (code: string) => void;
  disabled?: boolean;
}

const MultiLineComposer: React.FC<MultiLineComposerProps> = ({
  isOpen,
  onToggle,
  code,
  onChange,
  onRunCode,
  onStageToPrompt,
  disabled = false,
}) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [selectedSnippetIdx, setSelectedSnippetIdx] = useState<string>("");

  useEffect(() => {
    if (isOpen) {
      setTimeout(() => textareaRef.current?.focus(), 20);
    }
  }, [isOpen]);

  const lineCount = Math.max(1, code.split("\n").length);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      if (code.trim() && !disabled) {
        onRunCode(code);
      }
      return;
    }

    if (e.key === "Tab") {
      e.preventDefault();
      const target = e.currentTarget;
      const start = target.selectionStart;
      const end = target.selectionEnd;

      if (e.shiftKey) {
        // Unindent up to 4 leading spaces on current line
        const before = code.slice(0, start);
        const lineStart = before.lastIndexOf("\n") + 1;
        const linePrefix = code.slice(lineStart, start);
        const spacesToRemove = linePrefix.match(/^ {1,4}/)?.[0].length || 0;
        if (spacesToRemove > 0) {
          const next = code.slice(0, lineStart) + code.slice(lineStart + spacesToRemove);
          onChange(next);
          setTimeout(() => {
            target.selectionStart = target.selectionEnd = Math.max(lineStart, start - spacesToRemove);
          }, 0);
        }
      } else {
        const indent = "    ";
        const next = code.slice(0, start) + indent + code.slice(end);
        onChange(next);
        setTimeout(() => {
          target.selectionStart = target.selectionEnd = start + indent.length;
        }, 0);
      }
    }
  };

  const handleSnippetSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const idx = e.target.value;
    setSelectedSnippetIdx("");
    if (idx === "") return;
    const snippet = ELDRITCH_SNIPPETS[Number(idx)];
    if (snippet) {
      onChange(code.trim() ? `${code.trimEnd()}\n\n${snippet.code}` : snippet.code);
      setTimeout(() => textareaRef.current?.focus(), 0);
    }
  };

  if (!isOpen) {
    return (
      <div className="mt-2 flex items-center justify-between px-3 py-1.5 rounded-md bg-[#181b22] border border-[#2e3440] text-xs text-gray-400">
        <div className="flex items-center gap-2">
          <Code2 size={14} className="text-purple-400" />
          <span>Need to write or paste a multi-line Eldritch script, loop, or function?</span>
        </div>
        <button
          type="button"
          onClick={onToggle}
          className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-[#252a36] hover:bg-[#313848] text-gray-200 font-medium transition-colors"
        >
          <ChevronUp size={13} />
          Open Multi-Line Composer
        </button>
      </div>
    );
  }

  return (
    <div
      data-testid="multiline-composer"
      className="mt-2 flex flex-col rounded-md bg-[#161920] border border-[#343b4a] shadow-lg overflow-hidden"
    >
      {/* Composer Header */}
      <div className="flex flex-wrap items-center justify-between gap-2 px-3 py-1.5 bg-[#1d212b] border-b border-[#2e3440] text-xs">
        <div className="flex items-center gap-2">
          <Code2 size={14} className="text-purple-400" />
          <span className="font-semibold text-gray-200">Multi-Line Eldritch Composer</span>
          <span className="text-gray-500 hidden sm:inline">
            • <kbd className="px-1 py-0.5 rounded bg-black/40 text-gray-300 font-mono">Tab</kbd> indent •{" "}
            <kbd className="px-1 py-0.5 rounded bg-black/40 text-gray-300 font-mono">Ctrl+Enter</kbd> execute
          </span>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative flex items-center">
            <Sparkles size={12} className="text-purple-400 absolute left-2 pointer-events-none" />
            <select
              aria-label="Insert Eldritch snippet"
              value={selectedSnippetIdx}
              onChange={handleSnippetSelect}
              className="pl-6 pr-2 py-1 rounded bg-[#14161b] border border-[#343b4a] text-gray-200 text-xs focus:outline-none focus:border-purple-500"
            >
              <option value="">Insert Snippet...</option>
              {ELDRITCH_SNIPPETS.map((s, idx) => (
                <option key={s.label} value={idx}>
                  {s.label}
                </option>
              ))}
            </select>
          </div>

          <button
            type="button"
            onClick={() => onChange("")}
            disabled={!code}
            title="Clear editor"
            className="inline-flex items-center gap-1 px-2 py-1 rounded text-gray-400 hover:text-white hover:bg-[#2a2f3d] disabled:opacity-40 transition-colors"
          >
            <Trash2 size={12} />
            Clear
          </button>

          <button
            type="button"
            onClick={onToggle}
            aria-label="Collapse Multi-Line Composer"
            className="inline-flex items-center gap-1 px-2 py-1 rounded text-gray-400 hover:text-white hover:bg-[#2a2f3d] transition-colors"
          >
            <ChevronDown size={13} />
            Hide
          </button>
        </div>
      </div>

      {/* Editor Body with Line Numbers */}
      <div className="flex relative max-h-48 min-h-[108px] overflow-y-auto font-mono text-sm bg-[#12141a]">
        <div
          aria-hidden="true"
          className="select-none py-2.5 px-2.5 text-right bg-[#161922] text-gray-600 border-r border-[#262b36] min-w-[2.5rem] leading-5 text-xs"
        >
          {Array.from({ length: lineCount }).map((_, idx) => (
            <div key={idx}>{idx + 1}</div>
          ))}
        </div>
        <textarea
          ref={textareaRef}
          aria-label="Multi-line Eldritch script editor"
          value={code}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          rows={Math.min(8, Math.max(4, lineCount))}
          placeholder={`# Write multi-line Eldritch code here...\nfor f in file.list("/tmp"):\n    print(f["file_name"])`}
          className="w-full py-2 px-3 bg-transparent text-gray-100 placeholder-gray-600 focus:outline-none resize-y leading-5 font-mono text-xs sm:text-sm"
          spellCheck={false}
        />
      </div>

      {/* Composer Footer Actions */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-[#191d26] border-t border-[#2e3440] text-xs">
        <span className="text-gray-500">
          {lineCount} {lineCount === 1 ? "line" : "lines"} •{" "}
          {code.length} chars
        </span>
        <div className="flex items-center gap-2">
          <button
            type="button"
            disabled={!code.trim() || disabled}
            onClick={() => {
              if (!code.trim().includes("\n")) {
                onStageToPrompt(code.trim());
                onChange("");
              } else {
                onStageToPrompt(code);
              }
            }}
            className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-[#272c3a] hover:bg-[#32384a] text-gray-200 disabled:opacity-40 transition-colors"
            title="Copy single-line command into the terminal input prompt"
          >
            <CornerDownLeft size={12} />
            Stage to Prompt
          </button>
          <button
            type="button"
            disabled={!code.trim() || disabled}
            onClick={() => {
              onRunCode(code);
              onChange("");
            }}
            className="inline-flex items-center gap-1.5 px-3 py-1 rounded bg-purple-600 hover:bg-purple-500 text-white font-semibold disabled:opacity-40 transition-colors"
          >
            <Play size={12} />
            Run Script (Ctrl+Enter)
          </button>
        </div>
      </div>
    </div>
  );
};

export default MultiLineComposer;
