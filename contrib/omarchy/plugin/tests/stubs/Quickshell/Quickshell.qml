pragma Singleton
import QtQml
import qs.Commons

// Test stub of the Quickshell core module. Quickshell's native plugin is
// linked into the quickshell binary, so a plain QML runtime cannot import it;
// this stub serves the one call the widget makes: reading an env var.
QtObject {
  function env(name) {
    var value = TestControl.env[name]
    return value === undefined ? "" : value
  }
}
