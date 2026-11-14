import QtQuick
import QtTest
import qs.Commons

// Headless tests for the real ../BarWidget.qml.
//
// Quickshell's native plugin is linked into the quickshell binary, so a plain
// QML runtime cannot import Quickshell or Quickshell.Io. The stub modules in
// tests/stubs/ stand in for them and let a test inject the environment, the
// state file content and the exit code of the command probe, so the widget
// under test is the file that actually ships.
//
// The widget's show/hide decision is asserted through its label rather than
// through Item.visible: in a runtime without a window no item is ever
// effectively visible, so `visible` reads false for every item, including
// items that declare `visible: true`. BarWidget.qml binds
// `visible: labelText !== ""`, so a non-empty label is the widget asking to
// be shown.
TestCase {
  id: testCase
  name: "PomoGoBarWidget"

  Item {
    id: host
    width: 200
    height: 40
  }

  Loader {
    id: loader
    anchors.fill: parent
  }

  readonly property string runtimeDir: "/tmp/pomogo-widget-test"
  readonly property string statePath: runtimeDir + "/pomogo/state.json"

  function nowSecs() { return Math.floor(Date.now() / 1000) }

  // A state file shaped like the one the PomoGo TUI writes.
  function state(overrides) {
    var base = {
      "version": 2,
      "mode": "pomodoro",
      "state": "work",
      "session_type": "work",
      "ends_at": nowSecs() + 1500,
      "paused": false,
      "remaining_secs": 1500,
      "pid": 4242,
      "session_count": 3,
      "updated_at": nowSecs(),
      "started_at": nowSecs() - 600,
      "total_secs": 1500,
      "today_focus_secs": 3600,
      "daily_goal_secs": 14400
    }
    for (var key in overrides) base[key] = overrides[key]
    return base
  }

  function shows(widget) { return widget.labelText !== "" }

  // Creates the widget with the given command-probe exit code and state file
  // content. `probeExit` is the exit code the stub Process reports for the
  // command probe: 0 means the command was found, anything else means missing.
  function makeWidget(probeExit, files, settings) {
    TestControl.reset()
    TestControl.env = { "XDG_RUNTIME_DIR": runtimeDir }
    TestControl.defaultProcessExitCode = probeExit
    TestControl.files = files || {}

    loader.source = ""
    wait(10)
    loader.source = "../BarWidget.qml"
    wait(60)

    var widget = loader.item
    verify(widget !== null, "BarWidget.qml failed to load")
    widget.settings = settings || {}
    wait(20)
    return widget
  }

  function withState(overrides, probeExit, settings) {
    var files = {}
    files[statePath] = JSON.stringify(state(overrides))
    return makeWidget(probeExit === undefined ? 0 : probeExit, files, settings)
  }

  // ---------------------------------------------------------------------
  // Missing dependency: the widget must explain itself instead of vanishing.
  // ---------------------------------------------------------------------

  function test_missing_command_probes_for_pomogo() {
    var widget = makeWidget(127, {})
    verify(widget.pomogoProbed, "the command probe never completed")
    verify(!widget.pomogoInstalled, "a missing command was reported as installed")
    verify(TestControl.processCalls.join(" ").indexOf("pomogo") !== -1,
      "the probe did not look for the pomogo command: " + TestControl.processCalls.join(", "))
  }

  function test_missing_command_shows_diagnostic() {
    var widget = makeWidget(127, {})
    verify(shows(widget), "the widget asks for no label when the pomogo command is missing")
    verify(widget.tooltip.indexOf("pomogo") !== -1,
      "the tooltip does not name the missing command: " + widget.tooltip)
    verify(widget.tooltip.indexOf("not found on PATH") !== -1,
      "the tooltip does not say the command was not found: " + widget.tooltip)
  }

  function test_probe_looks_where_the_bar_looks() {
    // The bar runs widget commands through a login shell, so a probe that used
    // a plain shell would miss a command the bar itself can reach.
    var widget = makeWidget(127, {})
    var probe = TestControl.processCalls.join(" ")
    verify(probe.indexOf("bash") !== -1, "the probe does not use bash: " + probe)
    verify(probe.indexOf("-lc") !== -1, "the probe does not run a login shell: " + probe)
    verify(probe.indexOf("command -v pomogo") !== -1,
      "the probe does not look up the command: " + probe)
  }

  function test_missing_command_can_be_hidden() {
    var widget = makeWidget(127, {}, { "whenMissing": "Hide" })
    verify(!shows(widget), "the diagnostic ignores the whenMissing setting")
  }

  function test_live_session_wins_over_a_missing_probe() {
    // A live session proves the command is installed, so it must outrank a
    // probe that says otherwise.
    var widget = withState({}, 127)
    verify(shows(widget), "the widget hides a live session because the probe missed the command")
    verify(/^\S+ \d{2}:\d{2}$/.test(widget.labelText),
      "unexpected label for a live session: " + widget.labelText)
    verify(widget.tooltip.indexOf("not found on PATH") === -1,
      "a live session is reported as missing: " + widget.tooltip)
  }

  function test_installed_command_does_not_report_a_missing_dependency() {
    var widget = makeWidget(0, {})
    verify(widget.pomogoProbed, "the command probe never completed")
    verify(widget.pomogoInstalled, "an installed command was reported as missing")
    verify(widget.tooltip.indexOf("not found on PATH") === -1,
      "an installed command is reported as missing: " + widget.tooltip)
  }

  // ---------------------------------------------------------------------
  // Existing behaviour that must not regress.
  // ---------------------------------------------------------------------

  function test_running_session_shows_countdown() {
    var widget = withState({})
    verify(shows(widget), "the widget asks for no label during a running session")
    verify(/^\S+ \d{2}:\d{2}$/.test(widget.labelText),
      "unexpected label for a running session: " + widget.labelText)
  }

  function test_running_session_tooltip_reports_phase() {
    var widget = withState({})
    verify(widget.tooltip.indexOf("Focus") !== -1,
      "the tooltip does not report the phase: " + widget.tooltip)
  }

  function test_closed_pomogo_hides_the_widget() {
    var widget = makeWidget(0, {})
    verify(!shows(widget), "the widget asks for a label while PomoGo is closed")
  }

  function test_stale_state_hides_the_widget() {
    var widget = withState({ "updated_at": nowSecs() - 600 })
    verify(!shows(widget), "the widget keeps asking for a label for a stale timer")
  }

  function test_paused_session_is_reported_as_paused() {
    var widget = withState({ "paused": true })
    verify(shows(widget), "the widget asks for no label during a paused session")
    verify(widget.paused, "a paused session is not reported as paused")
  }

  function test_idle_session_shows_only_when_asked() {
    var hidden = withState({ "state": "idle" })
    verify(!shows(hidden), "an idle session is shown by default")

    var shown = withState({ "state": "idle" }, 0, { "whenIdle": "Show" })
    verify(shows(shown), "the whenIdle setting is ignored")
    verify(shown.labelText.indexOf(":") === -1, "an idle session shows a countdown")
  }

  function test_break_session_uses_the_break_glyph() {
    var work = withState({})
    var workGlyph = work.glyph
    var brk = withState({ "session_type": "short_break", "state": "short_break" })
    verify(shows(brk), "the widget asks for no label during a break")
    verify(brk.glyph !== workGlyph, "a break uses the same glyph as a focus session")
  }

  function test_nudge_replaces_the_glyph() {
    var plain = withState({})
    var plainGlyph = plain.glyph
    var nudge = withState({ "nudge": "water" })
    verify(nudge.glyph !== plainGlyph, "a body reminder does not change the glyph")
    verify(nudge.tooltip.indexOf("water") !== -1,
      "the tooltip does not report the body reminder: " + nudge.tooltip)
  }

  function test_task_name_is_shown_when_asked() {
    var hidden = withState({ "task": "Write the report" })
    verify(hidden.labelText.indexOf("Write the report") === -1,
      "the task name is shown without being asked for")

    var shown = withState({ "task": "Write the report" }, 0, { "showTask": "On" })
    verify(shown.labelText.indexOf("Write the report") !== -1,
      "the showTask setting is ignored: " + shown.labelText)
  }

  function test_vertical_layout_drops_the_seconds() {
    var widget = withState({})
    widget.bar = ({ "vertical": true })
    wait(20)
    verify(widget.labelText.indexOf("\n") !== -1,
      "the vertical layout does not stack the label: " + widget.labelText)
    verify(widget.labelText.indexOf(":") === -1,
      "the vertical layout still shows seconds: " + widget.labelText)
  }

  function test_icon_only_layout_drops_the_clock() {
    var widget = withState({}, 0, { "display": "Icon only" })
    verify(shows(widget), "the widget asks for no label in the icon-only layout")
    verify(widget.labelText.indexOf(":") === -1,
      "the icon-only layout still shows the clock: " + widget.labelText)
  }
}
