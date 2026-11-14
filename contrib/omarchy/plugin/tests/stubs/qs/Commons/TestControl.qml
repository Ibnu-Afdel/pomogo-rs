pragma Singleton
import QtQml

// Shared, injectable state for the stub Quickshell modules. Tests drive the
// widget through it instead of touching a real filesystem or a real process.
QtObject {
  // name -> value returned by Quickshell.env()
  property var env: ({})

  // absolute path -> file content the stub FileView serves
  property var files: ({})

  // joined process command -> exit code the stub Process reports
  property var processes: ({})

  // exit code reported for any command without an explicit entry
  property int defaultProcessExitCode: 0

  // every process command the widget asked to run, in order
  property var processCalls: []

  function reset() {
    env = ({})
    files = ({})
    processes = ({})
    defaultProcessExitCode = 0
    processCalls = []
  }
}
