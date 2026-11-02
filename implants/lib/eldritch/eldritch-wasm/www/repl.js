import init, { WasmRepl } from './pkg/eldritch_wasm.js';

const HISTORY_KEY = 'eldritch_repl_history';
const ISSUE_BASE_URL = 'https://github.com/spellshift/realm/issues/new?labels=eldritch,bug';

let repl = null;
let currentVersion = '0.4.1';

const widgetEl = document.getElementById('eldritch-repl-widget');
const viewportEl = document.getElementById('repl-viewport');
const termEl = document.getElementById('terminal');
const currentLineEl = document.getElementById('current-line');
const inputEl = document.getElementById('hidden-input');
const suggestionsEl = document.getElementById('suggestions');
const versionBadgeEl = document.getElementById('repl-version-badge');
const reportBtnEl = document.getElementById('report-issue-btn');
const clearBtnEl = document.getElementById('repl-clear-btn');
const resetBtnEl = document.getElementById('repl-reset-btn');
const fullscreenBtnEl = document.getElementById('repl-fullscreen-btn');

function escapeHtml(s) {
    return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

export function printLine(text, className = '') {
    if (!termEl) return;
    const lines = String(text).split('\n');
    lines.forEach((line) => {
        const div = document.createElement('div');
        div.className = 'line ' + className;
        div.textContent = line;
        termEl.appendChild(div);
    });
    scrollTerminalToBottom();
}

// Expose function called by Rust's extern "C" fn repl_print(s: &str)
window.repl_print = (s) => {
    printLine(s);
};

function scrollTerminalToBottom() {
    if (viewportEl) {
        viewportEl.scrollTop = viewportEl.scrollHeight;
    }
    if (document.body.classList.contains('eldritch-repl-standalone')) {
        window.scrollTo(0, document.body.scrollHeight);
    }
}

function focusInput() {
    if (!inputEl) return;
    try {
        inputEl.focus({ preventScroll: true });
    } catch {
        inputEl.focus();
    }
}

function updateVersionUI(version) {
    if (!version) return;
    currentVersion = String(version).replace(/^v/, '');
    if (versionBadgeEl) {
        versionBadgeEl.textContent = `v${currentVersion}`;
    }
    if (reportBtnEl) {
        const body = encodeURIComponent(`\n\n---\nEnvironment: Eldritch WASM REPL v${currentVersion}`);
        reportBtnEl.href = `${ISSUE_BASE_URL}&body=${body}`;
    }
}

async function resolveVersion() {
    try {
        const pkgUrl = new URL('./pkg/package.json', import.meta.url);
        const resp = await fetch(pkgUrl);
        if (resp.ok) {
            const pkg = await resp.json();
            if (pkg && pkg.version) {
                updateVersionUI(pkg.version);
            }
        }
    } catch {
        // Fallback to default version if package.json cannot be fetched
    }
}

function saveHistory() {
    if (!repl) return;
    try {
        const hist = repl.get_history();
        localStorage.setItem(HISTORY_KEY, JSON.stringify(hist));
    } catch (e) {
        console.error('Failed to save history', e);
    }
}

function loadSavedHistory() {
    if (!repl) return;
    try {
        const saved = localStorage.getItem(HISTORY_KEY);
        if (saved) {
            const hist = JSON.parse(saved);
            if (Array.isArray(hist)) {
                repl.load_history(hist);
            }
        }
    } catch (e) {
        console.error('Failed to load history', e);
    }
}

function positionSuggestions() {
    if (!suggestionsEl || suggestionsEl.style.display === 'none') return;

    const cursorSpan = currentLineEl ? currentLineEl.querySelector('.cursor') : null;
    const anchorEl = cursorSpan || currentLineEl;
    if (!anchorEl) return;

    const rect = anchorEl.getBoundingClientRect();
    const sugRect = suggestionsEl.getBoundingClientRect();
    const vw = window.innerWidth || document.documentElement.clientWidth;
    const vh = window.innerHeight || document.documentElement.clientHeight;

    let left = cursorSpan ? rect.left : rect.left + 32;
    const maxLeft = Math.max(8, vw - sugRect.width - 12);
    left = Math.max(8, Math.min(left, maxLeft));

    let top = rect.bottom + 4;
    if (top + sugRect.height > vh - 12 && rect.top > sugRect.height + 12) {
        top = rect.top - sugRect.height - 4;
    }

    suggestionsEl.style.left = `${Math.round(left)}px`;
    suggestionsEl.style.top = `${Math.round(top)}px`;
}

function acceptSuggestionIndex(idx, fallbackText) {
    if (!repl) return;
    const state = repl.get_state();
    if (state.suggestions && state.suggestions.length > idx) {
        const count = state.suggestions.length;
        const curIdx = state.suggestion_idx ?? 0;
        const steps = (idx - curIdx + count) % count;
        for (let i = 0; i < steps; i++) {
            repl.handle_key('ArrowDown', false, false, false, false);
        }
        repl.handle_key('Enter', false, false, false, false);
        render();
        return;
    }

    if (fallbackText) {
        const res = repl.handle_paste(fallbackText);
        if (res.echo) printLine(res.echo);
        if (res.output) printLine(res.output);
        render();
    }
}

export function render() {
    if (!repl || !currentLineEl) return;
    const state = repl.get_state();

    const promptSpan = `<span class="prompt">${escapeHtml(state.prompt)}</span>`;
    const buf = state.buffer;
    const cur = state.cursor;
    const safeCur = Math.min(Math.max(0, cur), buf.length);

    const before = buf.substring(0, safeCur);
    const charAtCursor = buf.substring(safeCur, safeCur + 1) || ' ';
    const after = buf.substring(safeCur + 1);

    let inputHtml =
        escapeHtml(before) +
        `<span class="cursor">${escapeHtml(charAtCursor)}</span>` +
        escapeHtml(after);

    if (state.prompt.length > 0) {
        const padding = ' '.repeat(state.prompt.length);
        inputHtml = inputHtml.replace(/\n/g, '\n' + padding);
    }

    currentLineEl.innerHTML = promptSpan + inputHtml;

    if (suggestionsEl) {
        if (state.suggestions && state.suggestions.length > 0) {
            suggestionsEl.innerHTML = '';
            state.suggestions.forEach((s, idx) => {
                const div = document.createElement('div');
                div.className = 'suggestion-item';
                if (state.suggestion_idx !== undefined && state.suggestion_idx === idx) {
                    div.classList.add('selected');
                }
                div.innerText = s;
                div.addEventListener('mousedown', (e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    acceptSuggestionIndex(idx, s);
                    focusInput();
                });
                div.addEventListener('click', (e) => {
                    e.stopPropagation();
                    focusInput();
                });
                suggestionsEl.appendChild(div);
            });

            scrollTerminalToBottom();
            suggestionsEl.style.display = 'block';
            positionSuggestions();

            const selectedEl = suggestionsEl.querySelector('.suggestion-item.selected');
            if (selectedEl && typeof selectedEl.scrollIntoView === 'function') {
                selectedEl.scrollIntoView({ block: 'nearest' });
            }
        } else {
            suggestionsEl.style.display = 'none';
            scrollTerminalToBottom();
        }
    } else {
        scrollTerminalToBottom();
    }
}

const pendingKeys = [];

export function processKey(key, ctrl = false, alt = false, meta = false, shift = false) {
    if (!repl) {
        pendingKeys.push([key, ctrl, alt, meta, shift]);
        return;
    }
    const result = repl.handle_key(key, ctrl, alt, meta, shift);

    if (result.clear && termEl) {
        termEl.innerHTML = '';
    }
    if (result.echo) {
        printLine(result.echo);
    }
    if (result.output) {
        printLine(result.output);
    }

    render();

    if (result.output || result.echo) {
        saveHistory();
    }
}

export function runSnippet(code) {
    if (!repl) return;
    // Clear any open suggestions and in-progress line before running snippet
    repl.handle_key('c', true, false, false, false);
    repl.handle_key('c', true, false, false, false);
    const normalized = code.endsWith('\n') ? code : code + '\n';
    const res = repl.handle_paste(normalized);
    if (res.echo) printLine(res.echo);
    if (res.output) printLine(res.output);
    render();
    saveHistory();
    focusInput();
}

function clearScreen() {
    if (repl) {
        processKey('l', true, false, false, false);
    } else if (termEl) {
        termEl.innerHTML = '';
    }
    focusInput();
}

function resetRepl() {
    repl = new WasmRepl();
    loadSavedHistory();
    if (termEl) {
        termEl.innerHTML = '';
    }
    printLine(`Eldritch v${currentVersion} WASM REPL (type help() or press Tab to autocomplete)`);
    render();
    focusInput();
}

function toggleFullscreen() {
    if (!widgetEl) return;
    const isFullscreen = widgetEl.classList.toggle('repl-fullscreen');
    if (fullscreenBtnEl) {
        fullscreenBtnEl.textContent = isFullscreen ? 'Exit Full Screen' : 'Full Screen';
    }
    render();
    focusInput();
}

function setupEventListeners() {
    const focusArea = viewportEl || document;
    focusArea.addEventListener('click', () => {
        if (window.getSelection && window.getSelection().toString().length > 0) {
            return;
        }
        focusInput();
    });

    if (inputEl) {
        inputEl.addEventListener('keydown', (e) => {
            if (['Shift', 'Control', 'Alt', 'Meta', 'CapsLock'].includes(e.key)) {
                return;
            }

            // Allow mobile IME / virtual keyboard events to flow to the 'input' handler
            if (e.key === 'Unidentified' || e.key === 'Process' || e.isComposing || e.keyCode === 229) {
                return;
            }

            // Allow native copy when text is selected, and native paste
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'v') {
                return;
            }
            if (
                (e.ctrlKey || e.metaKey) &&
                e.key.toLowerCase() === 'c' &&
                window.getSelection &&
                window.getSelection().toString().length > 0
            ) {
                return;
            }

            const isHandledSpecial = [
                'ArrowUp',
                'ArrowDown',
                'ArrowLeft',
                'ArrowRight',
                'Tab',
                'Backspace',
                'Delete',
                'Enter',
                'Home',
                'End',
            ].includes(e.key);

            const isPrintable = e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey;
            const isCtrlShortcut =
                e.ctrlKey && ['c', 'l', 'a', 'e', 'k', 'u', 'w', 'r', 'd', ' '].includes(e.key.toLowerCase());

            if (isHandledSpecial || isPrintable || isCtrlShortcut) {
                e.preventDefault();
                inputEl.value = '';
                processKey(e.key, e.ctrlKey, e.altKey, e.metaKey, e.shiftKey);
            }
        });

        // Mobile virtual keyboards often emit 'input' instead of specific keydown keys
        inputEl.addEventListener('input', (e) => {
            const inputType = e.inputType || '';
            if (inputType === 'deleteContentBackward') {
                processKey('Backspace', false, false, false, false);
                inputEl.value = '';
                return;
            }
            if (inputType === 'deleteContentForward') {
                processKey('Delete', false, false, false, false);
                inputEl.value = '';
                return;
            }
            if (inputType === 'insertLineBreak') {
                processKey('Enter', false, false, false, false);
                inputEl.value = '';
                return;
            }

            const data = e.data !== undefined && e.data !== null ? e.data : inputEl.value;
            if (data) {
                for (const ch of data) {
                    if (ch === '\n' || ch === '\r') {
                        processKey('Enter', false, false, false, false);
                    } else {
                        processKey(ch, false, false, false, false);
                    }
                }
            }
            inputEl.value = '';
        });

        inputEl.addEventListener('paste', (e) => {
            e.preventDefault();
            const text = (e.clipboardData || window.clipboardData).getData('text');
            if (text && repl) {
                const res = repl.handle_paste(text);
                if (res.echo) printLine(res.echo);
                if (res.output) printLine(res.output);
                render();
                if (res.echo || res.output) {
                    saveHistory();
                }
            }
        });
    }

    if (clearBtnEl) {
        clearBtnEl.addEventListener('click', (e) => {
            e.preventDefault();
            clearScreen();
        });
    }

    if (resetBtnEl) {
        resetBtnEl.addEventListener('click', (e) => {
            e.preventDefault();
            resetRepl();
        });
    }

    if (fullscreenBtnEl) {
        fullscreenBtnEl.addEventListener('click', (e) => {
            e.preventDefault();
            toggleFullscreen();
        });
    }

    // Bind touch / mobile toolbar buttons
    document.querySelectorAll('[data-repl-key]').forEach((btn) => {
        const trigger = (e) => {
            e.preventDefault();
            e.stopPropagation();
            const key = btn.getAttribute('data-repl-key');
            const ctrl = btn.getAttribute('data-repl-ctrl') === 'true';
            const shift = btn.getAttribute('data-repl-shift') === 'true';
            if (key) {
                processKey(key, ctrl, false, false, shift);
            }
            focusInput();
        };
        btn.addEventListener('mousedown', (e) => e.preventDefault());
        btn.addEventListener('click', trigger);
    });

    // Bind quick example snippet chips
    document.querySelectorAll('[data-repl-snippet]').forEach((chip) => {
        chip.addEventListener('click', (e) => {
            e.preventDefault();
            const snippet = chip.getAttribute('data-repl-snippet');
            if (snippet) {
                runSnippet(snippet);
            }
        });
    });

    window.addEventListener('resize', () => {
        positionSuggestions();
    });
    window.addEventListener('scroll', () => {
        positionSuggestions();
    }, { passive: true });
    if (viewportEl) {
        viewportEl.addEventListener('scroll', () => {
            positionSuggestions();
        }, { passive: true });
    }
}

async function main() {
    updateVersionUI(currentVersion);
    setupEventListeners();
    focusInput();

    await init();
    repl = new WasmRepl();
    loadSavedHistory();

    printLine(`Eldritch v${currentVersion} WASM REPL (type help() or press Tab to autocomplete)`);
    render();
    while (pendingKeys.length > 0) {
        const [k, c, a, m, s] = pendingKeys.shift();
        processKey(k, c, a, m, s);
    }
    focusInput();

    resolveVersion();
}

main();
