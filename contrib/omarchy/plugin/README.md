# PomoGo for the Omarchy bar

An [omarchy-shell](https://omarchy.org/) bar widget for
[PomoGo](https://github.com/Ibnu-Afdel/pomogo-rs), the terminal Pomodoro and
deep-focus timer.

It shows the running segment's countdown, dims while paused, switches its icon
when PomoGo asks you to rest your eyes, drink water or stretch, and hides when
PomoGo is closed. The tooltip shows the task and today's progress toward your
daily focus goal.

- click: open or focus PomoGo
- right click: start, pause or resume
- middle click: skip to the next segment

## Install

The widget needs the `pomogo` command, 3.0 or newer (`yay -S pomogo`, or see
the PomoGo README). PomoGo can install the widget itself:

```sh
pomogo omarchy install
```

Or add this repository with Omarchy's plugin manager:

```sh
omarchy plugin add https://github.com/Ibnu-Afdel/omarchy-pomogo.git --enable
```

Move it with `omarchy bar move pomogo.timer --section right`. Its settings
(show the task name, show it while PomoGo is idle) are in Omarchy's bar
settings.

## How it works

While the PomoGo TUI is open, it rewrites
`$XDG_RUNTIME_DIR/pomogo/state.json` every second. The widget reads that file
and hides itself when the updates stop. Clicks run `pomogo toggle`,
`pomogo skip` and `omarchy-launch-or-focus-tui pomogo`.

This repository is generated from `contrib/omarchy/plugin` in the PomoGo
repository. Send changes there.

## License

MIT
