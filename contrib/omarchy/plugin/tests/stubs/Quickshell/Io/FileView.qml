import QtQuick
import QtQml
import qs.Commons

// Test stub of Quickshell.Io.FileView. Serves file content from the shared
// test control instead of touching the filesystem.
QtObject {
  id: root

  property string path: ""
  property bool watchChanges: false
  property bool printErrors: true
  property string content: ""

  signal fileChanged()
  signal loaded()
  signal loadFailed()

  function text() { return root.content }

  function reload() { root.load() }

  function load() {
    var files = TestControl.files
    if (root.path !== undefined && files[root.path] !== undefined) {
      root.content = files[root.path]
      root.loaded()
    } else {
      root.content = ""
      root.loadFailed()
    }
  }

  Component.onCompleted: root.load()
}
