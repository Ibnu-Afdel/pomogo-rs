# Changelog

## 4.0.2

- Fixed: clicking the Omarchy widget opened a second PomoGo when the first
  was started by hand or inside tmux, and the widget then flickered between
  the two. The click now focuses the running PomoGo.
- Only one PomoGo runs at a time: launching it again switches to the
  running one.
- New `pomogo focus` brings the running PomoGo's terminal to the front,
  switching tmux to its pane first.

## 4.0.1

- Fixed: skipping a segment on autopilot kept the skipped segment's clock,
  so a break started by skipping focus early ran for whatever was left of
  the focus block. Skipped segments are now stored with the time actually
  spent.
- Fixed: closing the terminal window left PomoGo running in the background
  at full CPU, and `kill` couldn't stop it. It now exits within two seconds.
- The Omarchy widget has an icon-only option; hover it for the time left.

## 4.0.0

PomoGo becomes a focus companion: set it up once, then it runs your day.

- **Setup wizard.** The first launch (or `pomogo setup`) asks for your rhythm
  (Classic 25/5, Steady 50/10 or Deep 90/20), daily goal, autopilot, body
  reminders and notifications, and offers the bar widget on Omarchy.
- **Autopilot** is on by default: focus rolls into breaks and back.
- **Body reminders** while focusing: eyes every 20 minutes, water every 45,
  stretching every 60. Breaks reset the eye and stretch reminders. `w` logs a
  glass of water; `esc` dismisses a reminder.
- **Break tips**: each break suggests one thing to do.
- **Daily goal** (4 hours by default) with a notification when reached, and
  today's focus, streak and water on screen, in `pomogo stats`, in the Waybar
  tooltip and in the Omarchy widget.
- **New default `focus` layout** with tty-clock digits and context-aware key
  hints; `enter` starts, pauses and resumes.
- **Help overlay** grouped into Session, Companion, Look and General.
- The Omarchy widget swaps its icon during a reminder and shows today's
  progress in its tooltip.
- `pomogo screenshot-preview --scene ready|focus|reminder|break|deep` and
  `pomogo themes --json`, used to render the website from the real binary.
- The bar widget's repository now has a preview image and install, update
  and removal instructions for the Omarchy plugin marketplace.
- Fixed: finished segments were stored with the next segment's duration, so
  focus time was undercounted.

## 3.0.1

- `pomogo omarchy install` leaves a widget added with `omarchy plugin add`
  alone, and `uninstall` removes it through `omarchy plugin remove`.
- The bar widget is published on its own as `omarchy-pomogo`.

## 3.0.0

First release of the Rust rewrite: Omarchy 4 bar widget, live Omarchy
theming, pause on Hyprland lock, `pomogo toggle`/`skip`, `install.sh`, and an
AUR package. Reads the Go 2.x database, including its timestamps.
