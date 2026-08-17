---
trigger: always_on
---

# Visual UI Verification Rule
- Whenever launching, testing, or modifying desktop GUI applications (e.g. system tray daemons, flyouts, dialogs), you MUST visually verify the application using `computer_screenshot`.
- Never assume a UI is open, closed, or rendered correctly without taking a screenshot first.
