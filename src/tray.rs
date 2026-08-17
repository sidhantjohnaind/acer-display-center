#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(windows)]
mod win32_tray {
    use std::sync::atomic::{AtomicBool, Ordering};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM, S_OK};
    use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
    use windows_sys::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    };
    use windows_sys::Win32::Graphics::Gdi::{
        CreateBitmap, CreateCompatibleBitmap, CreateCompatibleDC, CreatePen, CreateSolidBrush,
        DeleteDC, DeleteObject, FillRect, GetDC, LineTo, MoveToEx, ReleaseDC, RoundRect,
        SelectObject, PS_SOLID,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
        DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW,
        LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow, TrackPopupMenuEx,
        TranslateMessage, HICON, HMENU, ICONINFO, IDI_APPLICATION, MF_POPUP, MF_SEPARATOR,
        MF_STRING, MSG, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_APP,
        WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK, WM_LBUTTONUP,
        WM_MBUTTONUP, WM_RBUTTONUP, WNDCLASSW,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL,
    };

    const WM_TRAY_CALLBACK: u32 = WM_APP + 101;
    static GUI_OPEN: AtomicBool = AtomicBool::new(false);
    static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

    unsafe fn create_custom_monitor_icon() -> HICON {
        let cx = 16;
        let cy = 16;
        let hdc_screen = GetDC(0 as _);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm_color = CreateCompatibleBitmap(hdc_screen, cx, cy);
        let hbm_old = SelectObject(hdc_mem, hbm_color as _);

        // Deep Obsidian background
        let rc = windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: cx, bottom: cy };
        let hbg = CreateSolidBrush(0x00100C0B);
        FillRect(hdc_mem, &rc, hbg);
        DeleteObject(hbg as _);

        // Glowing Neon Cyan Monitor Frame
        let hpen = CreatePen(PS_SOLID as i32, 1, 0x00F8BD38); // #38BDF8
        let hold_pen = SelectObject(hdc_mem, hpen as _);
        let h_mon_brush = CreateSolidBrush(0x001E1812);
        let hold_brush = SelectObject(hdc_mem, h_mon_brush as _);

        RoundRect(hdc_mem, 1, 1, 15, 12, 3, 3);

        // Stand and base
        MoveToEx(hdc_mem, 7, 12, std::ptr::null_mut());
        LineTo(hdc_mem, 9, 12);
        MoveToEx(hdc_mem, 4, 14, std::ptr::null_mut());
        LineTo(hdc_mem, 12, 14);

        SelectObject(hdc_mem, hold_brush);
        DeleteObject(h_mon_brush as _);
        SelectObject(hdc_mem, hold_pen);
        DeleteObject(hpen as _);
        SelectObject(hdc_mem, hbm_old);
        DeleteDC(hdc_mem);
        ReleaseDC(0 as _, hdc_screen);

        let hbm_mask = CreateBitmap(cx, cy, 1, 1, std::ptr::null());
        let ii = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let h_icon = CreateIconIndirect(&ii);
        DeleteObject(hbm_color as _);
        DeleteObject(hbm_mask as _);

        if h_icon != 0 as _ {
            h_icon
        } else {
            LoadIconW(0 as _, IDI_APPLICATION)
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct KBDLLHOOKSTRUCT {
        vk_code: u32,
        scan_code: u32,
        flags: u32,
        time: u32,
        dw_extra_info: usize,
    }

    static mut HOOK_HANDLE: windows_sys::Win32::UI::WindowsAndMessaging::HHOOK = 0 as _;

    unsafe extern "system" fn low_level_keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 && (wparam as u32 == 0x0100 /* WM_KEYDOWN */ || wparam as u32 == 0x0104 /* WM_SYSKEYDOWN */) {
            if HOTKEYS_ENABLED.load(Ordering::SeqCst) {
                let kbd = *(lparam as *const KBDLLHOOKSTRUCT);

                let ctrl = (windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x11 /* VK_CONTROL */) as u16 & 0x8000) != 0;
                let alt = (windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x12 /* VK_MENU */) as u16 & 0x8000) != 0;
                let shift = (windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x10 /* VK_SHIFT */) as u16 & 0x8000) != 0;
                let win = ((windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x5B /* VK_LWIN */) as u16 & 0x8000) != 0)
                    || ((windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x5C /* VK_RWIN */) as u16 & 0x8000) != 0);

                let config = crate::hotkeys::HotkeyConfig::load();
                for binding in &config.bindings {
                    if binding.mod_ctrl == ctrl
                        && binding.mod_alt == alt
                        && binding.mod_shift == shift
                        && binding.mod_win == win
                        && binding.win32_vk() == kbd.vk_code
                    {
                        let cmd = binding.command.clone();
                        std::thread::spawn(move || {
                            if cmd.first().map(|s| s.as_str()) == Some("gui") {
                                spawn_gui();
                            } else {
                                let _ = crate::cli::dispatch_command(cmd);
                            }
                        });
                        return 1; // Handled
                    }
                }
            }
        }
        windows_sys::Win32::UI::WindowsAndMessaging::CallNextHookEx(HOOK_HANDLE, code, wparam, lparam)
    }

    pub fn spawn_gui() {
        std::thread::spawn(|| {
            unsafe {
                let title = to_wide("Acer Display Center");
                let hwnd = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(std::ptr::null(), title.as_ptr());
                if hwnd != 0 as _ {
                    windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, 9 /* SW_RESTORE */);
                    windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
                    return;
                }
            }

            let exe = std::env::current_exe().unwrap_or_else(|_| {
                std::path::PathBuf::from(r"C:\Users\Admin\AppData\Local\Programs\acer_monitor_cli\amctl.exe")
            });
            let _ = std::process::Command::new(&exe)
                .arg("gui")
                .spawn();
        });
    }

    fn to_wide(s: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new(s).encode_wide().chain(Some(0)).collect()
    }

    pub static HOTKEYS_ENABLED: AtomicBool = AtomicBool::new(true);

    pub fn set_hotkeys_enabled(enabled: bool) {
        HOTKEYS_ENABLED.store(enabled, Ordering::SeqCst);
    }

    pub fn are_hotkeys_enabled() -> bool {
        HOTKEYS_ENABLED.load(Ordering::SeqCst)
    }

    #[derive(Debug, Clone)]
    pub struct TrayMonitorState {
        pub brightness: u32,
        pub contrast: u32,
        pub volume: u32,
        pub is_muted: bool,
        pub preset: String,
        pub overdrive: String,
        pub aimpoint: u32,
        pub input: String,
        pub bluelight: u32,
        pub colortemp: String,
        pub gamma: String,
        pub colorspace: String,
        pub black_boost: u32,
        pub hdr: bool,
    }

    impl Default for TrayMonitorState {
        fn default() -> Self {
            Self {
                brightness: 75,
                contrast: 50,
                volume: 50,
                is_muted: false,
                preset: "Standard".into(),
                overdrive: "Off".into(),
                aimpoint: 0,
                input: "DP".into(),
                bluelight: 0,
                colortemp: "Normal".into(),
                gamma: "2.2".into(),
                colorspace: "sRGB".into(),
                black_boost: 5,
                hdr: false,
            }
        }
    }

    static CURRENT_STATE: std::sync::Mutex<Option<TrayMonitorState>> = std::sync::Mutex::new(None);

    fn probe_state_background() {
        std::thread::spawn(|| {
            let mut st = TrayMonitorState::default();
            if let Ok(mut set) = crate::monitor::MonitorSet::enumerate() {
                if let Some(mon) = set.monitors_mut().first_mut() {
                    if let Ok((b, _)) = mon.get_vcp(0x10) { st.brightness = b; }
                    if let Ok((c, _)) = mon.get_vcp(0x12) { st.contrast = c; }
                    if let Ok((v, _)) = mon.get_vcp(0x62) { st.volume = v; }
                    if let Ok((m, _)) = mon.get_vcp(0x8D) { st.is_muted = m == 1; }
                    if let Ok((dm, _)) = mon.get_vcp(0xE2) {
                        st.preset = match dm {
                            0 => "User",
                            1 => "Standard",
                            2 => "ECO",
                            3 => "Graphics",
                            4 => "Movie",
                            5 => "Action",
                            6 => "Racing",
                            7 => "Sports",
                            11 => "HDR",
                            _ => "Standard",
                        }.to_string();
                    }
                    if let Ok((inp, _)) = mon.get_vcp(0x60) {
                        st.input = match inp {
                            0x0F => "DP",
                            0x11 => "HDMI 1",
                            0x12 => "HDMI 2",
                            _ => "AUTO",
                        }.to_string();
                    }
                    if let Ok((od, _)) = crate::acer::get_overdrive(mon).or_else(|_| mon.get_vcp(0x92)) {
                        st.overdrive = match od {
                            0 => "Off",
                            1 => "Normal",
                            2 => "Extreme",
                            _ => "Off",
                        }.to_string();
                    }
                    if let Ok((aim, _)) = crate::acer::get_aim_type(mon) {
                        st.aimpoint = aim;
                    }
                    if let Ok((bl, _)) = crate::acer::get_blue_light(mon) {
                        st.bluelight = bl;
                    }
                    if let Ok((ct, _)) = crate::acer::get_color_temp(mon) {
                        st.colortemp = match ct {
                            0 => "Cool",
                            1 => "Normal",
                            2 => "Warm",
                            3 => "BlueLight",
                            4 => "User",
                            _ => "Normal",
                        }.to_string();
                    }
                }
            }
            st.hdr = crate::hdr::get_os_hdr();
            if let Ok(mut guard) = CURRENT_STATE.lock() {
                *guard = Some(st);
            }
        });
    }

    unsafe fn show_context_menu(hwnd: HWND) {
        let hmenu: HMENU = CreatePopupMenu();
        if hmenu == 0 as _ {
            return;
        }

        let st = if let Ok(guard) = CURRENT_STATE.lock() {
            guard.clone().unwrap_or_default()
        } else {
            TrayMonitorState::default()
        };

        let p_label = |name: &str, cur_name: &str, title: &str| -> String {
            if cur_name.eq_ignore_ascii_case(name) {
                format!("● {} (Current)", title)
            } else {
                format!("   {}", title)
            }
        };

        let b_label = |target: u32, cur: u32, title: &str| -> String {
            if cur == target {
                format!("● {} (Current)", title)
            } else {
                format!("   {}", title)
            }
        };

        // 1. Open Flyout
        AppendMenuW(hmenu, MF_STRING, 100, to_wide("🚀 Open Acer Display Center (Ctrl+Alt+M)").as_ptr());
        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());

        // 2. Picture Presets Submenu
        let m_presets = CreatePopupMenu();
        AppendMenuW(m_presets, MF_STRING, 201, to_wide(&p_label("action", &st.preset, "⚔️ Action (Gaming)")).as_ptr());
        AppendMenuW(m_presets, MF_STRING, 202, to_wide(&p_label("racing", &st.preset, "🏎️ Racing")).as_ptr());
        AppendMenuW(m_presets, MF_STRING, 203, to_wide(&p_label("sports", &st.preset, "⚽ Sports")).as_ptr());
        AppendMenuW(m_presets, MF_STRING, 204, to_wide(&p_label("standard", &st.preset, "⚡ Standard")).as_ptr());
        AppendMenuW(m_presets, MF_STRING, 205, to_wide(&p_label("eco", &st.preset, "🌱 ECO Mode (Ctrl+Alt+E)")).as_ptr());
        AppendMenuW(m_presets, MF_STRING, 206, to_wide(&p_label("movie", &st.preset, "🎬 Movie")).as_ptr());
        AppendMenuW(m_presets, MF_STRING, 207, to_wide(&p_label("graphics", &st.preset, "🎨 Graphics / sRGB")).as_ptr());
        AppendMenuW(m_presets, MF_STRING, 208, to_wide(&p_label("hdr", &st.preset, "✨ HDR Game (Hardware)")).as_ptr());
        AppendMenuW(m_presets, MF_STRING, 209, to_wide(&p_label("user", &st.preset, "👤 User Custom")).as_ptr());
        let presets_title = format!("🎮 Picture Presets (Current: {})", st.preset);
        AppendMenuW(hmenu, MF_POPUP, m_presets as usize, to_wide(&presets_title).as_ptr());

        // 3. Brightness Submenu
        let m_bright = CreatePopupMenu();
        AppendMenuW(m_bright, MF_STRING, 301, to_wide(&b_label(100, st.brightness, "☀️ 100% (Maximum)")).as_ptr());
        AppendMenuW(m_bright, MF_STRING, 302, to_wide(&b_label(75, st.brightness, "☀️ 75%")).as_ptr());
        AppendMenuW(m_bright, MF_STRING, 303, to_wide(&b_label(50, st.brightness, "☀️ 50% (Balanced)")).as_ptr());
        AppendMenuW(m_bright, MF_STRING, 304, to_wide(&b_label(25, st.brightness, "☀️ 25%")).as_ptr());
        AppendMenuW(m_bright, MF_STRING, 305, to_wide(&b_label(10, st.brightness, "🌙 10% (Night Dim)")).as_ptr());
        AppendMenuW(m_bright, MF_STRING, 306, to_wide(&b_label(0, st.brightness, "🌙 0% (Minimum)")).as_ptr());
        AppendMenuW(m_bright, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_bright, MF_STRING, 307, to_wide("⬆️ Brightness +10% (Ctrl+Alt+Up)").as_ptr());
        AppendMenuW(m_bright, MF_STRING, 308, to_wide("⬇️ Brightness -10% (Ctrl+Alt+Down)").as_ptr());
        let bright_title = format!("☀️ Brightness (Current: {}%)", st.brightness);
        AppendMenuW(hmenu, MF_POPUP, m_bright as usize, to_wide(&bright_title).as_ptr());

        // 4. Contrast Submenu
        let m_contrast = CreatePopupMenu();
        AppendMenuW(m_contrast, MF_STRING, 401, to_wide(&b_label(80, st.contrast, "Contrast 80%")).as_ptr());
        AppendMenuW(m_contrast, MF_STRING, 402, to_wide(&b_label(70, st.contrast, "Contrast 70%")).as_ptr());
        AppendMenuW(m_contrast, MF_STRING, 403, to_wide(&b_label(60, st.contrast, "Contrast 60%")).as_ptr());
        AppendMenuW(m_contrast, MF_STRING, 404, to_wide(&b_label(50, st.contrast, "Contrast 50% (Default)")).as_ptr());
        AppendMenuW(m_contrast, MF_STRING, 405, to_wide(&b_label(40, st.contrast, "Contrast 40%")).as_ptr());
        let contrast_title = format!("🌓 Contrast (Current: {}%)", st.contrast);
        AppendMenuW(hmenu, MF_POPUP, m_contrast as usize, to_wide(&contrast_title).as_ptr());

        // 5. Gaming & Esports Submenu
        let m_gaming = CreatePopupMenu();
        let aim_str = match st.aimpoint {
            0 => "Off",
            1 => "Red Dot",
            2 => "Crosshair 1",
            3 => "Crosshair 2",
            _ => "Off",
        };
        AppendMenuW(m_gaming, MF_STRING, 501, to_wide("🎯 AimPoint: Cycle Next").as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 502, to_wide(if st.aimpoint == 0 { "● 🎯 AimPoint: Off (Current)" } else { "   🎯 AimPoint: Off" }).as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 503, to_wide(if st.aimpoint == 1 { "● 🎯 AimPoint: Red Dot (Current)" } else { "   🎯 AimPoint: Red Dot" }).as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 504, to_wide(if st.aimpoint == 2 { "● 🎯 AimPoint: Crosshair 1 (Current)" } else { "   🎯 AimPoint: Crosshair 1" }).as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 505, to_wide(if st.aimpoint == 3 { "● 🎯 AimPoint: Crosshair 2 (Current)" } else { "   🎯 AimPoint: Crosshair 2" }).as_ptr());
        AppendMenuW(m_gaming, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_gaming, MF_STRING, 506, to_wide("📊 Toggle Refresh Rate / FPS HUD").as_ptr());
        AppendMenuW(m_gaming, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_gaming, MF_STRING, 507, to_wide(&p_label("extreme", &st.overdrive, "⚡ OverDrive: Extreme")).as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 508, to_wide(&p_label("normal", &st.overdrive, "⚡ OverDrive: Normal")).as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 509, to_wide(&p_label("off", &st.overdrive, "⚡ OverDrive: Off")).as_ptr());
        AppendMenuW(m_gaming, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_gaming, MF_STRING, 510, to_wide(&b_label(0, st.black_boost, "🌑 Black Boost: 0 (Off)")).as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 511, to_wide(&b_label(5, st.black_boost, "🌑 Black Boost: 5 (Standard)")).as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 512, to_wide(&b_label(8, st.black_boost, "🌑 Black Boost: 8 (Enhanced)")).as_ptr());
        AppendMenuW(m_gaming, MF_STRING, 513, to_wide(&b_label(10, st.black_boost, "🌑 Black Boost: 10 (Maximum)")).as_ptr());
        let gaming_title = format!("🎯 Gaming & Esports ({}, Aim: {})", st.overdrive, aim_str);
        AppendMenuW(hmenu, MF_POPUP, m_gaming as usize, to_wide(&gaming_title).as_ptr());

        // 6. Color & Eye Shield Submenu
        let m_color = CreatePopupMenu();
        AppendMenuW(m_color, MF_STRING, 601, to_wide(if st.bluelight == 0 { "● 🛡️ Blue Light: Off (Current)" } else { "   🛡️ Blue Light: Off" }).as_ptr());
        AppendMenuW(m_color, MF_STRING, 602, to_wide(if st.bluelight == 1 { "● 🛡️ Blue Light: 50% (Current)" } else { "   🛡️ Blue Light: 50%" }).as_ptr());
        AppendMenuW(m_color, MF_STRING, 603, to_wide(if st.bluelight == 2 { "● 🛡️ Blue Light: 60% (Current)" } else { "   🛡️ Blue Light: 60%" }).as_ptr());
        AppendMenuW(m_color, MF_STRING, 604, to_wide(if st.bluelight == 3 { "● 🛡️ Blue Light: 70% (Current)" } else { "   🛡️ Blue Light: 70%" }).as_ptr());
        AppendMenuW(m_color, MF_STRING, 605, to_wide(if st.bluelight == 4 { "● 🛡️ Blue Light: 80% (Current)" } else { "   🛡️ Blue Light: 80%" }).as_ptr());
        AppendMenuW(m_color, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_color, MF_STRING, 606, to_wide(&p_label("warm", &st.colortemp, "🌡️ Color Temp: Warm")).as_ptr());
        AppendMenuW(m_color, MF_STRING, 607, to_wide(&p_label("normal", &st.colortemp, "🌡️ Color Temp: Normal")).as_ptr());
        AppendMenuW(m_color, MF_STRING, 608, to_wide(&p_label("cool", &st.colortemp, "🌡️ Color Temp: Cool")).as_ptr());
        AppendMenuW(m_color, MF_STRING, 609, to_wide(&p_label("bluelight", &st.colortemp, "🌡️ Color Temp: BlueLight")).as_ptr());
        AppendMenuW(m_color, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_color, MF_STRING, 610, to_wide(&p_label("2.2", &st.gamma, "📐 Gamma: 2.2 (Default)")).as_ptr());
        AppendMenuW(m_color, MF_STRING, 611, to_wide(&p_label("2.4", &st.gamma, "📐 Gamma: 2.4 (Darker)")).as_ptr());
        AppendMenuW(m_color, MF_STRING, 612, to_wide(&p_label("2.0", &st.gamma, "📐 Gamma: 2.0 (Brighter)")).as_ptr());
        AppendMenuW(m_color, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_color, MF_STRING, 613, to_wide(&p_label("srgb", &st.colorspace, "🎨 Color Space: sRGB")).as_ptr());
        AppendMenuW(m_color, MF_STRING, 614, to_wide(&p_label("dcip3", &st.colorspace, "🎨 Color Space: DCI-P3")).as_ptr());
        AppendMenuW(m_color, MF_STRING, 615, to_wide(&p_label("rec709", &st.colorspace, "🎨 Color Space: Rec.709")).as_ptr());
        let color_title = format!("🎨 Color & Eye Shield ({})", st.colortemp);
        AppendMenuW(hmenu, MF_POPUP, m_color as usize, to_wide(&color_title).as_ptr());

        // 7. Video Input Submenu
        let m_input = CreatePopupMenu();
        AppendMenuW(m_input, MF_STRING, 701, to_wide(&p_label("dp", &st.input, "🔌 DisplayPort (DP)")).as_ptr());
        AppendMenuW(m_input, MF_STRING, 702, to_wide(&p_label("hdmi 1", &st.input, "🔌 HDMI 1")).as_ptr());
        AppendMenuW(m_input, MF_STRING, 703, to_wide(&p_label("hdmi 2", &st.input, "🔌 HDMI 2")).as_ptr());
        AppendMenuW(m_input, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_input, MF_STRING, 704, to_wide("🔄 Auto Select Input").as_ptr());
        AppendMenuW(m_input, MF_STRING, 705, to_wide("⏭️ Next Input").as_ptr());
        let input_title = format!("🔌 Input Source (Current: {})", st.input);
        AppendMenuW(hmenu, MF_POPUP, m_input as usize, to_wide(&input_title).as_ptr());

        // 8. Audio Submenu
        let m_audio = CreatePopupMenu();
        let mute_label = if st.is_muted { "🔇 Unmute Audio (Currently MUTED)" } else { "🔇 Mute Audio" };
        AppendMenuW(m_audio, MF_STRING, 801, to_wide(mute_label).as_ptr());
        AppendMenuW(m_audio, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_audio, MF_STRING, 802, to_wide(&b_label(100, st.volume, "🔊 Volume 100%")).as_ptr());
        AppendMenuW(m_audio, MF_STRING, 803, to_wide(&b_label(75, st.volume, "🔊 Volume 75%")).as_ptr());
        AppendMenuW(m_audio, MF_STRING, 804, to_wide(&b_label(50, st.volume, "🔊 Volume 50%")).as_ptr());
        AppendMenuW(m_audio, MF_STRING, 805, to_wide(&b_label(25, st.volume, "🔊 Volume 25%")).as_ptr());
        AppendMenuW(m_audio, MF_STRING, 806, to_wide(&b_label(0, st.volume, "🔊 Volume 0%")).as_ptr());
        AppendMenuW(m_audio, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_audio, MF_STRING, 807, to_wide("🔊 Volume +10%").as_ptr());
        AppendMenuW(m_audio, MF_STRING, 808, to_wide("🔉 Volume -10%").as_ptr());
        let audio_title = if st.is_muted { "🔊 Audio (Current: MUTED)".into() } else { format!("🔊 Audio (Current: {}%)", st.volume) };
        AppendMenuW(hmenu, MF_POPUP, m_audio as usize, to_wide(&audio_title).as_ptr());

        // 9. Hardware & Power Tools Submenu
        let m_tools = CreatePopupMenu();
        let hdr_label = if st.hdr { "✨ Toggle Unified HDR (Currently ON)" } else { "✨ Toggle Unified HDR (Currently OFF)" };
        AppendMenuW(m_tools, MF_STRING, 901, to_wide(hdr_label).as_ptr());
        AppendMenuW(m_tools, MF_STRING, 902, to_wide("🔄 Sync All Displays").as_ptr());
        AppendMenuW(m_tools, MF_STRING, 903, to_wide("🔒 Lock Physical OSD Keys").as_ptr());
        AppendMenuW(m_tools, MF_STRING, 904, to_wide("🔓 Unlock Physical OSD Keys").as_ptr());
        AppendMenuW(m_tools, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_tools, MF_STRING, 905, to_wide("📐 Display Calibration Grid").as_ptr());
        AppendMenuW(m_tools, MF_STRING, 906, to_wide("⚡ Energy Estimate").as_ptr());
        AppendMenuW(m_tools, MF_STRING, 907, to_wide("📋 Run Diagnostic Scan").as_ptr());
        AppendMenuW(m_tools, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(m_tools, MF_STRING, 908, to_wide("⚠️ Factory Reset Monitor").as_ptr());
        AppendMenuW(m_tools, MF_STRING, 909, to_wide("🌙 Turn Display Off (DDC/CI)").as_ptr());
        AppendMenuW(hmenu, MF_POPUP, m_tools as usize, to_wide("🛠️ Hardware Tools & Power").as_ptr());

        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());

        // 10. Global Hotkeys Toggle
        let hk_label = if HOTKEYS_ENABLED.load(Ordering::SeqCst) {
            "✔ Global Hotkeys (Enabled)"
        } else {
            "✖ Global Hotkeys (Disabled)"
        };
        AppendMenuW(hmenu, MF_STRING, 105, to_wide(hk_label).as_ptr());
        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());

        // 11. Exit
        AppendMenuW(hmenu, MF_STRING, 999, to_wide("❌ Exit Tray Daemon").as_ptr());

        let mut pt: POINT = std::mem::zeroed();
        GetCursorPos(&mut pt);

        SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenuEx(
            hmenu,
            TPM_RETURNCMD | TPM_BOTTOMALIGN | TPM_LEFTALIGN | TPM_RIGHTBUTTON,
            pt.x,
            pt.y,
            hwnd,
            std::ptr::null(),
        );
        DestroyMenu(hmenu);

        fn show_info_box(title: &str, msg: &str) {
            let title = title.to_string();
            let msg = msg.to_string();
            std::thread::spawn(move || {
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe)
                        .args(["report", &title, &msg])
                        .spawn();
                }
            });
        }

        fn run_cli(args: &[&str]) {
            let cmd: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            std::thread::spawn(move || {
                let _ = crate::cli::dispatch_command(cmd);
                probe_state_background();
            });
        }

        fn run_cli_with_notify(args: &[&str], title: &'static str) {
            let cmd: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            std::thread::spawn(move || {
                let result = crate::cli::dispatch_command(cmd);
                probe_state_background();
                match result {
                    Ok(msg) if !msg.is_empty() => {
                        show_info_box(title, &msg);
                    }
                    Err(e) => {
                        show_info_box("Error", &e);
                    }
                    _ => {}
                }
            });
        }

        match cmd {
            100 => spawn_gui(),
            105 => {
                let cur = HOTKEYS_ENABLED.load(Ordering::SeqCst);
                HOTKEYS_ENABLED.store(!cur, Ordering::SeqCst);
            }

            // Presets
            201 => run_cli(&["preset", "action"]),
            202 => run_cli(&["preset", "racing"]),
            203 => run_cli(&["preset", "sports"]),
            204 => run_cli(&["preset", "standard"]),
            205 => run_cli(&["preset", "eco"]),
            206 => run_cli(&["preset", "movie"]),
            207 => run_cli(&["preset", "graphics"]),
            208 => run_cli(&["preset", "hdr"]),
            209 => run_cli(&["preset", "user"]),

            // Brightness
            301 => run_cli(&["brightness", "100", "--osd"]),
            302 => run_cli(&["brightness", "75", "--osd"]),
            303 => run_cli(&["brightness", "50", "--osd"]),
            304 => run_cli(&["brightness", "25", "--osd"]),
            305 => run_cli(&["brightness", "10", "--osd"]),
            306 => run_cli(&["brightness", "0", "--osd"]),
            307 => run_cli(&["brightness", "+10", "--osd"]),
            308 => run_cli(&["brightness", "-10", "--osd"]),

            // Contrast
            401 => run_cli(&["contrast", "80"]),
            402 => run_cli(&["contrast", "70"]),
            403 => run_cli(&["contrast", "60"]),
            404 => run_cli(&["contrast", "50"]),
            405 => run_cli(&["contrast", "40"]),

            // Gaming
            501 => run_cli(&["aimpoint", "next"]),
            502 => run_cli(&["aimpoint", "0"]),
            503 => run_cli(&["aimpoint", "1"]),
            504 => run_cli(&["aimpoint", "2"]),
            505 => run_cli(&["aimpoint", "3"]),
            506 => run_cli(&["hz", "toggle"]),
            507 => run_cli(&["overdrive", "extreme"]),
            508 => run_cli(&["overdrive", "normal"]),
            509 => run_cli(&["overdrive", "off"]),
            510 => run_cli(&["blackboost", "0"]),
            511 => run_cli(&["blackboost", "5"]),
            512 => run_cli(&["blackboost", "8"]),
            513 => run_cli(&["blackboost", "10"]),

            // Color & Eye Shield
            601 => run_cli(&["bluelight", "0"]),
            602 => run_cli(&["bluelight", "1"]),
            603 => run_cli(&["bluelight", "2"]),
            604 => run_cli(&["bluelight", "3"]),
            605 => run_cli(&["bluelight", "4"]),
            606 => run_cli(&["colortemp", "warm"]),
            607 => run_cli(&["colortemp", "normal"]),
            608 => run_cli(&["colortemp", "cool"]),
            609 => run_cli(&["colortemp", "bluelight"]),
            610 => run_cli(&["gamma", "2.2"]),
            611 => run_cli(&["gamma", "2.4"]),
            612 => run_cli(&["gamma", "2.0"]),
            613 => run_cli(&["colorspace", "srgb"]),
            614 => run_cli(&["colorspace", "dcip3"]),
            615 => run_cli(&["colorspace", "rec709"]),

            // Input
            701 => run_cli(&["input", "dp"]),
            702 => run_cli(&["input", "hdmi1"]),
            703 => run_cli(&["input", "hdmi2"]),
            704 => run_cli(&["input", "auto"]),
            705 => run_cli(&["input", "next"]),

            // Audio
            801 => run_cli(&["mute", "toggle"]),
            802 => run_cli(&["volume", "100"]),
            803 => run_cli(&["volume", "75"]),
            804 => run_cli(&["volume", "50"]),
            805 => run_cli(&["volume", "25"]),
            806 => run_cli(&["volume", "0"]),
            807 => run_cli(&["volume", "+10"]),
            808 => run_cli(&["volume", "-10"]),

            // Hardware & Tools
            901 => run_cli_with_notify(&["hdr", "both", "toggle"], "✨ Unified HDR Bridge"),
            902 => run_cli_with_notify(&["sync"], "🔄 Display Sync"),
            903 => run_cli_with_notify(&["keylock", "on"], "🔒 Physical OSD Keylock"),
            904 => run_cli_with_notify(&["unlock"], "🔓 Physical OSD Unlock"),
            905 => run_cli_with_notify(&["test-pattern", "grid"], "📐 Display Calibration Pattern"),
            906 => run_cli_with_notify(&["energy"], "⚡ Energy Consumption Estimate"),
            907 => run_cli_with_notify(&["diag"], "📋 Hardware Diagnostic Scan"),
            908 => {
                std::thread::spawn(|| {
                    unsafe {
                        let res = windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                            0 as _,
                            to_wide("Are you sure you want to restore the monitor to factory default settings?").as_ptr(),
                            to_wide("⚠️ Factory Reset Monitor").as_ptr(),
                            windows_sys::Win32::UI::WindowsAndMessaging::MB_YESNO | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONWARNING | windows_sys::Win32::UI::WindowsAndMessaging::MB_TOPMOST,
                        );
                        if res == 6 /* IDYES */ {
                            let _ = crate::cli::dispatch_command(vec!["reset".into()]);
                            show_info_box("Factory Reset", "Monitor has been restored to factory defaults.");
                        }
                    }
                });
            }
            909 => run_cli_with_notify(&["power", "off"], "🌙 Display Power Control"),

            999 => {
                EXIT_REQUESTED.store(true, Ordering::SeqCst);
                PostQuitMessage(0);
            }
            _ => {}
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_TRAY_CALLBACK => {
                let event = (lparam & 0xFFFF) as u32;
                match event {
                    WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                        spawn_gui();
                        0
                    }
                    WM_RBUTTONUP => {
                        show_context_menu(hwnd);
                        0
                    }
                    _ => 0,
                }
            }

            WM_HOTKEY => {
                if !HOTKEYS_ENABLED.load(Ordering::SeqCst) {
                    return 0;
                }
                let id = wparam as usize;
                let config = crate::hotkeys::HotkeyConfig::load();
                if id >= 1 && id <= config.bindings.len() {
                    let binding = config.bindings[id - 1].clone();
                    std::thread::spawn(move || {
                        if binding.command.first().map(|s| s.as_str()) == Some("gui") {
                            spawn_gui();
                        } else {
                            let _ = crate::cli::dispatch_command(binding.command);
                        }
                    });
                }
                0
            }

            WM_DESTROY | WM_CLOSE => {
                EXIT_REQUESTED.store(true, Ordering::SeqCst);
                PostQuitMessage(0);
                0
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub fn run_tray() -> Result<(), String> {
        println!("Starting Pure Rust Acer Monitor System Tray Daemon (amctl tray)...");

        unsafe {
            let hr = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
            if hr != S_OK && hr != 1 {
                eprintln!("Note: CoInitializeEx returned 0x{:08X}", hr);
            }

            let class_name = to_wide("AcerPureRustTrayClass");
            let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());

            let hicon = create_custom_monitor_icon();

            let wnd_class = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: hicon,
                hCursor: 0 as _,
                hbrBackground: 0 as _,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };
            RegisterClassW(&wnd_class);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                to_wide("AcerTray").as_ptr(),
                0,
                0, 0, 0, 0,
                0 as HWND,
                0 as _,
                hinstance,
                std::ptr::null(),
            );

            if hwnd == 0 as _ {
                return Err(format!("Failed to create tray window: {}", windows_sys::Win32::Foundation::GetLastError()));
            }

            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1001;
            nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            nid.uCallbackMessage = WM_TRAY_CALLBACK;
            nid.hIcon = hicon;

            let tip = "Acer Monitor Control (amctl)\nSingle/Double-Click: Open Quick Settings\nRight-Click: Menu";
            let tip_wide = to_wide(tip);
            let len = tip_wide.len().min(nid.szTip.len() - 1);
            nid.szTip[..len].copy_from_slice(&tip_wide[..len]);

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);

            println!("● 100% Pure Standalone Rust Binary active.");
            println!("● Left-Click / Double-Click -> Opens Frame Studio Quick Settings GUI");
            println!("● Right-Click -> Shows Menu");
            println!("● Global Hotkeys active (Loaded from hotkeys.json):");

            let config = crate::hotkeys::HotkeyConfig::load();
            for (idx, binding) in config.bindings.iter().enumerate() {
                let id = (idx + 1) as i32;
                let mods = binding.win32_modifiers();
                let vk = binding.win32_vk();
                let _ = RegisterHotKey(hwnd, id, mods, vk);
                println!("   [{}] {:<22} -> {}", binding.to_display_string(), binding.name, binding.description);
            }

            HOOK_HANDLE = windows_sys::Win32::UI::WindowsAndMessaging::SetWindowsHookExW(
                13, // WH_KEYBOARD_LL
                Some(low_level_keyboard_proc),
                hinstance,
                0,
            );

            let mut msg: MSG = std::mem::zeroed();
            while !EXIT_REQUESTED.load(Ordering::SeqCst) {
                let ret = GetMessageW(&mut msg, 0 as _, 0, 0);
                if ret <= 0 {
                    if EXIT_REQUESTED.load(Ordering::SeqCst) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    continue;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if HOOK_HANDLE != 0 as _ {
                windows_sys::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx(HOOK_HANDLE);
                HOOK_HANDLE = 0 as _;
            }

            for i in 1..=32 {
                UnregisterHotKey(hwnd, i);
            }
            Shell_NotifyIconW(NIM_DELETE, &nid);
            DestroyWindow(hwnd);

            Ok(())
        }
    }
}

#[cfg(windows)]
pub use win32_tray::*;

#[cfg(not(windows))]
pub fn run_tray() -> Result<(), String> {
    Err("System tray is only supported on Windows".into())
}

#[cfg(not(windows))]
pub fn spawn_gui() {}
