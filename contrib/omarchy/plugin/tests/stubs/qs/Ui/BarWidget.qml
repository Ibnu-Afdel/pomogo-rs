import QtQuick

// Test stub of qs.Ui.BarWidget. Mirrors the contract plugin widgets rely on:
// the host-injected properties and the setting() helper.
Item {
  id: root

  // `var` rather than `QtObject` so a test can inject a plain object as the
  // host bar and read `vertical` / `barSize` off it.
  property var bar: null
  property string moduleName: ""
  property var settings: ({})

  readonly property bool vertical: bar ? bar.vertical : false
  readonly property int barSize: bar && bar.barSize !== undefined ? bar.barSize : 30

  function setting(name, fallback) {
    var value = settings ? settings[name] : undefined
    return value === undefined || value === null ? fallback : value
  }
}
