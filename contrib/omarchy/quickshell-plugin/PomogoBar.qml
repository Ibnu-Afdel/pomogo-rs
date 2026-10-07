import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

// Native Quickshell Bar Widget for Pomogo on Omarchy Linux.
// Reads real-time focus state directly from $XDG_RUNTIME_DIR/pomogo/state.json

Scope {
    id: root

    property string stateFilePath: Quickshell.env("XDG_RUNTIME_DIR") ? Quickshell.env("XDG_RUNTIME_DIR") + "/pomogo/state.json" : "/tmp/pomogo/state.json"
    property var pomogoState: null
    property bool hasActiveSession: pomogoState !== null && pomogoState.state !== "idle"

    FileWatcher {
        path: root.stateFilePath
        onFileChanged: {
            readFile();
        }
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            readFile();
        }
    }

    function readFile() {
        // Read JSON state file
        try {
            var content = Quickshell.readFile(root.stateFilePath);
            if (content && content.length > 0) {
                root.pomogoState = JSON.parse(content);
            } else {
                root.pomogoState = null;
            }
        } catch (e) {
            root.pomogoState = null;
        }
    }

    Component.onCompleted: {
        readFile();
    }

    // Top-bar pill widget component
    RowLayout {
        spacing: 6
        visible: root.hasActiveSession

        Rectangle {
            id: pill
            implicitHeight: 28
            implicitWidth: contentRow.implicitWidth + 16
            radius: 14
            color: root.pomogoState && root.pomogoState.session_type === "work" ? "#1e1e2e" : "#282a36"
            border.color: root.pomogoState && root.pomogoState.session_type === "work" ? "#f7768e" : "#7aa2f7"
            border.width: 1

            RowLayout {
                id: contentRow
                anchors.centerIn: parent
                spacing: 6

                Text {
                    text: {
                        if (!root.pomogoState) return "🍅";
                        if (root.pomogoState.paused) return "⏸️";
                        if (root.pomogoState.session_type === "work") return "🍅";
                        return "☕";
                    }
                    font.pixelSize: 13
                }

                Text {
                    text: {
                        if (!root.pomogoState) return "00:00";
                        var totalSecs = root.pomogoState.remaining_secs || 0;
                        var mins = Math.floor(totalSecs / 60);
                        var secs = totalSecs % 60;
                        return (mins < 10 ? "0" : "") + mins + ":" + (secs < 10 ? "0" : "") + secs;
                    }
                    font.family: "JetBrains Mono"
                    font.pixelSize: 12
                    font.bold: true
                    color: "#c0caf5"
                }

                Text {
                    visible: root.pomogoState && root.pomogoState.task && root.pomogoState.task.length > 0
                    text: root.pomogoState && root.pomogoState.task ? "· " + root.pomogoState.task : ""
                    font.pixelSize: 11
                    color: "#7f849c"
                    elide: Text.ElideRight
                    Layout.maximumWidth: 120
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    // Launch or focus terminal with pomogo
                    Quickshell.execDetached(["alacritty", "-e", "pomogo"]);
                }
            }
        }
    }
}

