---
name: windows-computer-use
description: >-
  Standard operating procedures for inspecting, automating, and verifying Windows
  desktop applications and UI using Windows Computer Use MCP tools.
---

# Windows Computer Use Workflow Guide

## Available MCP Tools (`windows-computer-use`)
Call these lazy-loaded tools via `call_mcp_tool` with `ServerName: "windows-computer-use"`:
- `computer_screenshot`: Capture full Windows desktop display.
- `computer_list_windows`: Enumerate all running application windows and handles.
- `computer_launch_app`: Launch an application or script by executable name or path.
- `computer_focus_window`: Bring a window to the active foreground.
- `computer_click`: Click at specific (x, y) coordinates on screen.
- `computer_type_text`: Type text into the active focused window.
- `computer_press_key`: Send special keys (Enter, Escape, Tab, Alt+F4, etc.).
- `computer_close_window`: Gracefully close a target window.

## Standard Execution Loop
1. **Locate & Focus**:
   - Call `computer_list_windows` or `computer_focus_window` to ensure the target window is active.
2. **Visual Observation**:
   - Always call `computer_screenshot` first. Never interact blindly without observing current visual state.
3. **Execute Interaction**:
   - Call `computer_click`, `computer_type_text`, or `computer_press_key` with target parameters.
4. **Verification**:
   - Take a subsequent `computer_screenshot` to confirm the desired visual change or UI update occurred.
