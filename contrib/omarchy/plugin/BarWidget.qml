import QtQuick
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

// PomoGo countdown for the Omarchy bar.
//
// The TUI rewrites $XDG_RUNTIME_DIR/pomogo/state.json every second while it
// is open. The widget shows the running segment from that file and hides
// itself once the heartbeat stops, so a closed or crashed TUI never leaves a
// frozen timer behind.
//
// Left click opens (or focuses) PomoGo, right click starts/pauses/resumes,
// middle click skips to the next segment.
BarWidget {
  id: root
  moduleName: "pomogo.timer"

  readonly property string runtimeDir: Quickshell.env("XDG_RUNTIME_DIR") || "/tmp"
  readonly property string statePath: runtimeDir + "/pomogo/state.json"

  // The TUI writes once a second; allow a few missed beats before hiding.
  readonly property int staleAfterSecs: 4

  // The widget needs the `pomogo` command: the TUI is what writes the state
  // file and what the clicks drive. Probe for the command so a missing
  // install is reported in the bar instead of leaving an empty slot that
  // looks like a broken widget.
  property bool pomogoInstalled: true
  property bool pomogoProbed: false

  readonly property bool missing: pomogoProbed && !pomogoInstalled
  readonly property bool showWhenMissing: String(setting("whenMissing", "Show")) === "Show"

  property var pomo: null
  property real now: Date.now() / 1000

  readonly property bool alive: pomo !== null && (now - (pomo.updated_at || 0)) <= staleAfterSecs
  readonly property bool idle: !alive || pomo.state === "idle" || pomo.state === ""
  readonly property bool paused: alive && pomo.paused === true
  readonly property bool onBreak: alive && !idle && pomo.session_type !== "work"
  readonly property bool showTask: String(setting("showTask", "Off")) === "On"
  readonly property bool showWhenIdle: String(setting("whenIdle", "Hide")) === "Show"
  readonly property bool iconOnly: String(setting("display", "Icon and time")) === "Icon only"

  readonly property int remainingSecs: {
    if (!alive || idle) return 0
    var deep = pomo.mode === "deep" && (pomo.block_ends_at || pomo.block_remaining_secs)
    var endsAt = deep ? pomo.block_ends_at : pomo.ends_at
    var stored = deep ? pomo.block_remaining_secs : pomo.remaining_secs
    if (!paused && endsAt > 0) return Math.max(0, Math.round(endsAt - now))
    return Math.max(0, stored || 0)
  }

  // A body reminder from the TUI (eyes, water, stretch), if one is showing.
  readonly property string nudge: alive && pomo.nudge ? String(pomo.nudge) : ""
  readonly property var nudgeGlyphs: ({ eyes: "󰈈", water: "󰖌", stretch: "󰖃" })
  readonly property var nudgeTitles: ({ eyes: "Rest your eyes", water: "Drink some water", stretch: "Stand up and stretch" })

  // Nerd Font glyphs, matching the rest of the Omarchy bar.
  readonly property string glyph: nudge !== "" && nudgeGlyphs[nudge] ? nudgeGlyphs[nudge]
    : idle ? "󰔛" : paused ? "󰏤" : onBreak ? "󰅶" : "󰔟"

  // A package that is not there. Only used when nothing is running, so a live
  // session always keeps its own glyph.
  readonly property string missingGlyph: "󰅚"

  function hm(secs) {
    var mins = Math.floor(Math.max(0, secs || 0) / 60)
    var h = Math.floor(mins / 60)
    var m = mins % 60
    return h === 0 ? m + "m" : m === 0 ? h + "h" : h + "h " + m + "m"
  }

  readonly property string todayText: alive && pomo.daily_goal_secs > 0
    ? "Today: " + hm(pomo.today_focus_secs) + " of " + hm(pomo.daily_goal_secs)
    : alive && pomo.today_focus_secs > 0 ? "Today: " + hm(pomo.today_focus_secs) : ""

  readonly property string clockText: {
    var s = remainingSecs
    var h = Math.floor(s / 3600)
    var m = Math.floor((s % 3600) / 60)
    var sec = s % 60
    var mm = (m < 10 ? "0" : "") + m
    var ss = (sec < 10 ? "0" : "") + sec
    return h > 0 ? h + ":" + mm + ":" + ss : mm + ":" + ss
  }

  readonly property string taskText: alive && pomo.task ? String(pomo.task) : ""

  readonly property string labelText: {
    // A live session always wins. The TUI is what writes the state file, so a
    // session proves the command is installed even when the probe misses it,
    // which happens when the bar's PATH differs from the interactive shell's.
    if (!alive) return missing && showWhenMissing ? missingGlyph : ""
    if (idle) return showWhenIdle ? glyph : ""
    if (iconOnly) return glyph
    if (vertical) return glyph + "\n" + clockText.split(":").slice(-2)[0]
    var label = glyph + " " + clockText
    if (showTask && taskText !== "") label += "  " + (taskText.length > 24 ? taskText.slice(0, 23) + "…" : taskText)
    return label
  }

  readonly property string tooltip: {
    if (!alive) {
      if (!missing) return ""
      return [
        "PomoGo was not found on PATH",
        "The bar could not find the pomogo command",
        "Install it where the bar can reach it",
        "https://github.com/Ibnu-Afdel/pomogo-rs#install"
      ].join("\n")
    }
    if (idle) return ["PomoGo is ready", todayText, "Right click to start"].filter(function(l) { return l !== "" }).join("\n")
    var phase = pomo.session_type === "work" ? "Focus" : pomo.session_type === "long_break" ? "Long break" : "Break"
    var lines = []
    if (nudge !== "" && nudgeTitles[nudge]) lines.push(nudgeTitles[nudge])
    lines.push(phase + (paused ? " (paused)" : "") + " · " + clockText + " left")
    if (pomo.project_name) lines.push("Project: " + pomo.project_name)
    if (taskText !== "") lines.push("Task: " + taskText)
    if (todayText !== "") lines.push(todayText)
    lines.push("Click to open · right click to " + (paused ? "resume" : "pause") + " · middle click to skip")
    return lines.join("\n")
  }

  function parse(content) {
    try {
      var parsed = JSON.parse(String(content || ""))
      root.pomo = parsed && typeof parsed === "object" ? parsed : null
    } catch (e) {
      // A half-written file is replaced atomically a second later.
    }
  }

  function run(command) {
    if (root.bar) root.bar.run(command)
  }

  visible: labelText !== ""
  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  FileView {
    id: stateFile
    path: root.statePath
    watchChanges: true
    printErrors: false
    onFileChanged: reload()
    onLoaded: root.parse(text())
    onLoadFailed: root.pomo = null
  }

  // Drives the countdown between writes and re-reads the file, since the
  // atomic rename can outrun a file watch.
  Timer {
    interval: 1000
    running: true
    repeat: true
    triggeredOnStart: true
    onTriggered: {
      root.now = Date.now() / 1000
      stateFile.reload()
    }
  }

  // Looks for the `pomogo` command. The bar runs widget commands through a
  // login shell (see qs.Commons.Util.execDetached), so the probe has to look
  // in the same place: a plain shell would miss a command that only the
  // login PATH carries.
  Process {
    id: probe
    command: ["bash", "-lc", "command -v pomogo >/dev/null 2>&1"]
    onExited: function(exitCode) {
      root.pomogoInstalled = exitCode === 0
      root.pomogoProbed = true
    }
  }

  // Probes at startup and every half minute, so a command installed while the
  // bar is running is picked up without restarting the shell.
  Timer {
    interval: 30000
    running: true
    repeat: true
    triggeredOnStart: true
    onTriggered: if (!probe.running) probe.running = true
  }

  WidgetButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: root.labelText
    dimmed: root.paused
    tooltipText: root.tooltip
    horizontalMargin: 8.75
    verticalPadding: 8.75

    onPressed: function(b) {
      if (b === Qt.RightButton) root.run("pomogo toggle")
      else if (b === Qt.MiddleButton) root.run("pomogo skip")
      // `pomogo focus` finds PomoGo's terminal even when it was started by
      // hand or inside tmux; launch a new one only when none is running.
      else root.run("pomogo focus || omarchy-launch-or-focus-tui pomogo")
    }
  }
}
