import QtQuick
import QtQml
import qs.Commons

// Test stub of Quickshell.Io.Process. Reports an exit code from the shared
// test control instead of starting a real process.
QtObject {
  id: root

  property bool running: false
  property var command: []

  signal exited(int exitCode, int exitStatus)

  onRunningChanged: if (root.running) root.start()

  function start() {
    var key = Array.isArray(root.command) ? root.command.join(" ") : String(root.command)
    TestControl.processCalls.push(key)
    var code = TestControl.processes[key]
    if (code === undefined) code = TestControl.defaultProcessExitCode
    root.exited(code, 0)
    root.running = false
  }
}
