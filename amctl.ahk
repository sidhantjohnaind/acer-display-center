; ==============================================================================
; Acer Monitor Control (amctl) Windows AutoHotkey (AHK) Global Hotkeys
; Press Win+Alt+Up / Down for Brightness, Win+Alt+Left / Right for Presets, etc.
; ==============================================================================

#NoEnv
#SingleInstance Force
SetWorkingDir %A_ScriptDir%

; Win+Alt+Up -> Brightness +10%
#!Up::
Run, acer_monitor_cli.exe brightness +10 --osd, , Hide
return

; Win+Alt+Down -> Brightness -10%
#!Down::
Run, acer_monitor_cli.exe brightness -10 --osd, , Hide
return

; Win+Alt+VolumeUp -> Volume +5%
#!Volume_Up::
Run, acer_monitor_cli.exe volume +5 --osd, , Hide
return

; Win+Alt+VolumeDown -> Volume -5%
#!Volume_Down::
Run, acer_monitor_cli.exe volume -5 --osd, , Hide
return

; Win+Alt+M -> Mute Toggle
#!m::
Run, acer_monitor_cli.exe mute toggle --osd, , Hide
return

; Win+Alt+H -> HDR Gaming Mode Preset
#!h::
Run, acer_monitor_cli.exe preset hdr, , Hide
return

; Win+Alt+E -> ECO Power Saver Mode Preset
#!e::
Run, acer_monitor_cli.exe preset eco, , Hide
return

; Win+Alt+S -> Standard Mode Preset
#!s::
Run, acer_monitor_cli.exe preset standard, , Hide
return

; Win+Alt+D -> Apply Day Mode
#!d::
Run, acer_monitor_cli.exe brightness 90 --osd, , Hide
return

; Win+Alt+N -> Apply Night Mode
#!n::
Run, acer_monitor_cli.exe brightness 20 --osd, , Hide
return
