---
title: Interactive
tags:
 - User Guide
description: Interactive WASM Eldritch REPL
permalink: user-guide/eldritch-interactive
---

<link rel="stylesheet" href="{{ site.baseurl }}/assets/eldritch-repl/repl.css">

# Interactive Eldritch REPL

Test [Eldritch]({{ site.baseurl }}/user-guide/eldritch) expressions, standard library functions, and tome snippets directly in your browser. This interactive environment runs the Eldritch interpreter compiled to WebAssembly (WASM) with simulated system, file, process, network, and agent bindings.

<div class="repl-examples" aria-label="Quick example snippets">
  <button type="button" class="repl-example-chip" data-repl-snippet="sys.info()">sys.info()</button>
  <button type="button" class="repl-example-chip" data-repl-snippet="file.list()">file.list()</button>
  <button type="button" class="repl-example-chip" data-repl-snippet="process.list()">process.list()</button>
  <button type="button" class="repl-example-chip" data-repl-snippet="net.interfaces()">net.interfaces()</button>
  <button type="button" class="repl-example-chip" data-repl-snippet="user.info()">user.info()</button>
  <button type="button" class="repl-example-chip" data-repl-snippet='hash.sha256("realm")'>hash.sha256("realm")</button>
</div>

<div id="eldritch-repl-widget" class="repl-widget repl-embedded">
  <div class="repl-topbar">
    <div class="repl-topbar-left">
      <span class="repl-window-dots">
        <span class="repl-dot repl-dot-red"></span>
        <span class="repl-dot repl-dot-yellow"></span>
        <span class="repl-dot repl-dot-green"></span>
      </span>
      <span class="repl-title">Eldritch WASM REPL</span>
      <span id="repl-version-badge" class="repl-version-badge">v0.4.1</span>
    </div>
    <div id="header-controls" class="repl-topbar-actions">
      <button type="button" id="repl-clear-btn" class="repl-btn" title="Clear terminal output (Ctrl+L)">Clear</button>
      <button type="button" id="repl-reset-btn" class="repl-btn" title="Reset interpreter state">Reset</button>
      <button type="button" id="repl-fullscreen-btn" class="repl-btn" title="Toggle full screen">Full Screen</button>
      <a id="report-issue-btn"
         href="https://github.com/spellshift/realm/issues/new?labels=eldritch,bug"
         target="_blank" rel="noopener noreferrer" class="repl-btn">
        Leave Feedback
      </a>
    </div>
  </div>

  <div id="repl-viewport" class="repl-viewport">
    <div id="terminal"></div>
    <div id="current-line"></div>
    <div id="suggestions"></div>
    <input type="text" id="hidden-input" autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" aria-label="Eldritch REPL input">
  </div>

  <div id="repl-mobile-toolbar" class="repl-toolbar" role="toolbar" aria-label="REPL quick keys">
    <span class="repl-toolbar-label">Keys:</span>
    <button type="button" class="repl-key-btn" data-repl-key="Tab" title="Autocomplete (Tab)">Tab ⇥</button>
    <button type="button" class="repl-key-btn" data-repl-key="ArrowUp" title="Previous command / suggestion (Up)">↑</button>
    <button type="button" class="repl-key-btn" data-repl-key="ArrowDown" title="Next command / suggestion (Down)">↓</button>
    <button type="button" class="repl-key-btn" data-repl-key="ArrowLeft" title="Move cursor left">←</button>
    <button type="button" class="repl-key-btn" data-repl-key="ArrowRight" title="Move cursor right">→</button>
    <button type="button" class="repl-key-btn" data-repl-key="(" title="Insert (">(</button>
    <button type="button" class="repl-key-btn" data-repl-key=")" title="Insert )">)</button>
    <button type="button" class="repl-key-btn" data-repl-key='"' title='Insert "'>"</button>
    <button type="button" class="repl-key-btn" data-repl-key="." title="Insert .">.</button>
    <button type="button" class="repl-key-btn" data-repl-key="c" data-repl-ctrl="true" title="Cancel current line (Ctrl+C)">Ctrl+C</button>
    <button type="button" class="repl-key-btn repl-key-btn-primary" data-repl-key="Enter" title="Execute line (Enter)">Run ↵</button>
  </div>
</div>

<script type="module" src="{{ site.baseurl }}/assets/eldritch-repl/repl.js"></script>

## Keyboard & Touch Controls

| Shortcut / Action | Description |
| :--- | :--- |
| `Enter` / **Run ↵** | Execute the current line or submit a completed multi-line block |
| `Shift + Enter` | Force execution of the current multi-line block |
| `Tab` / `Ctrl + Space` | Trigger autocomplete for built-ins, standard library modules, and methods |
| `↑` / `↓` | Navigate command history (or cycle through autocomplete suggestions) |
| `Ctrl + R` | Reverse-search through command history |
| `Ctrl + C` | Cancel the current input line or close suggestions |
| `Ctrl + L` / **Clear** | Clear the terminal screen |
