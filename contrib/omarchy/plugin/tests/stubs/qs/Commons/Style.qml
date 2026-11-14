pragma Singleton
import QtQml

// Test stub of qs.Commons.Style. The widget only needs Style.bar.sizeHorizontal
// through the base class fallback.
QtObject {
  property var bar: ({ "sizeHorizontal": 30 })
}
