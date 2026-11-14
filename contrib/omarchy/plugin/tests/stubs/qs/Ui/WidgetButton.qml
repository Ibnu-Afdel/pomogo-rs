import QtQuick

// Test stub of qs.Ui.WidgetButton: the properties the widget sets, plus the
// pressed(int) signal the widget listens to.
Item {
  id: root

  property var bar: null
  property string text: ""
  property bool dimmed: false
  property string tooltipText: ""
  property real horizontalMargin: 8.5
  property real verticalPadding: 6

  signal pressed(int button)

  // Lets a test fire the same signal the real button emits on a click.
  function click(button) { root.pressed(button) }
}
