#!/usr/bin/env python3
"""
Types a text character by character into the first text field of the
window 'genc³wb - Input' at a fixed frequency.

Install dependencies once:
    pip install pyautogui pyperclip pygetwindow
    (Linux additionally: sudo apt install xdotool)

Run:
    python type_into_genc3.py
"""

import platform
import subprocess
import sys
import time

import pyautogui
import pyperclip

# ---------------------------------------------------------------- settings --
WINDOW_TITLE = "genc³wb - Input"
FREQUENCY_HZ = 1.5                 # characters per second
TABS_TO_FIRST_FIELD = 0            # press Tab this many times after focusing
START_DELAY_S = 3.0                # time to switch on your screen recorder

TEXT = """Vom Eise befreit sind Strom und Bäche,
Durch des Frühlings holden, belebenden Blick,
Im Tale grünet Hoffnungs-Glück;
Der alte Winter, in seiner Schwäche,
Zog sich in rauhe Berge zurück."""
# -----------------------------------------------------------------------------

INTERVAL = 1.0 / FREQUENCY_HZ
PASTE_HOTKEY = ("command", "v") if platform.system() == "Darwin" else ("ctrl", "v")


def focus_window(title: str) -> None:
    system = platform.system()
    if system == "Linux":
        # xdotool handles X11; search is case-insensitive substring match
        try:
            subprocess.run(
                ["xdotool", "search", "--name", title, "windowactivate", "--sync"],
                check=True, timeout=5,
            )
        except (subprocess.CalledProcessError, FileNotFoundError):
            sys.exit(f"Window '{title}' not found (or xdotool missing).")
    else:
        import pygetwindow as gw  # Windows + macOS
        matches = [w for w in gw.getWindowsWithTitle(title)]
        if not matches:
            sys.exit(f"Window '{title}' not found.")
        win = matches[0]
        try:
            win.activate()
        except Exception:
            # macOS sometimes needs this instead of activate()
            win.minimize(); win.restore()
    time.sleep(0.5)


def type_char(ch: str) -> None:
    if ch == "\n":
        pyautogui.press("enter")
    elif ch.isascii() and ch.isprintable():
        pyautogui.write(ch)
    else:
        # non-ASCII (ä ö ü ß …): paste via clipboard
        pyperclip.copy(ch)
        pyautogui.hotkey(*PASTE_HOTKEY)


def main() -> None:
    print(f"Typing {len(TEXT)} characters at {FREQUENCY_HZ} Hz "
          f"({INTERVAL:.3f} s per character), "
          f"total ≈ {len(TEXT) * INTERVAL:.1f} s.")
    print(f"Starting in {START_DELAY_S:.0f} s – start your screen recording now.")
    time.sleep(START_DELAY_S)

    focus_window(WINDOW_TITLE)
    for _ in range(TABS_TO_FIRST_FIELD):
        pyautogui.press("tab")
        time.sleep(0.1)

    next_tick = time.perf_counter()
    for ch in TEXT:
        type_char(ch)
        next_tick += INTERVAL          # drift-free timing
        delay = next_tick - time.perf_counter()
        if delay > 0:
            time.sleep(delay)
    print("Done.")


if __name__ == "__main__":
    pyautogui.FAILSAFE = True          # move mouse to a screen corner to abort
    main()
