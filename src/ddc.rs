#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::{ffi::c_void, os::raw::c_char, ptr};

pub type BOOL = i32;
pub type DWORD = u32;
pub type HANDLE = *mut c_void;
pub type HMONITOR = HANDLE;
pub type HDC = HANDLE;
pub type LPARAM = isize;
pub type LONG = i32;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PHYSICAL_MONITOR {
    pub hPhysicalMonitor: HANDLE,
    pub szPhysicalMonitorDescription: [u16; 128],
}

impl Default for PHYSICAL_MONITOR {
    fn default() -> Self {
        Self {
            hPhysicalMonitor: ptr::null_mut(),
            szPhysicalMonitorDescription: [0u16; 128],
        }
    }
}

pub type MONITORENUMPROC = Option<unsafe extern "system" fn(HMONITOR, HDC, *mut RECT, LPARAM) -> BOOL>;

#[cfg(windows)]
#[link(name = "user32")]
extern "system" {
    pub fn EnumDisplayMonitors(
        hdc: HDC,
        lprcClip: *const RECT,
        lpfnEnum: MONITORENUMPROC,
        dwData: LPARAM,
    ) -> BOOL;
}

#[cfg(windows)]
#[link(name = "dxva2")]
extern "system" {
    pub fn GetNumberOfPhysicalMonitorsFromHMONITOR(
        hMonitor: HMONITOR,
        pdwNumberOfPhysicalMonitors: *mut DWORD,
    ) -> BOOL;

    pub fn GetPhysicalMonitorsFromHMONITOR(
        hMonitor: HMONITOR,
        dwPhysicalMonitorArraySize: DWORD,
        pPhysicalMonitorArray: *mut PHYSICAL_MONITOR,
    ) -> BOOL;

    pub fn DestroyPhysicalMonitors(
        dwPhysicalMonitorArraySize: DWORD,
        pPhysicalMonitorArray: *mut PHYSICAL_MONITOR,
    ) -> BOOL;

    pub fn SetVCPFeature(hMonitor: HANDLE, bVCPCode: u8, dwNewValue: DWORD) -> BOOL;

    pub fn GetVCPFeatureAndVCPFeatureReply(
        hMonitor: HANDLE,
        bVCPCode: u8,
        pvct: *mut DWORD,
        pvcp: *mut DWORD,
    ) -> BOOL;

    pub fn GetCapabilitiesStringLength(hMonitor: HANDLE, pdwCapabilitiesStringLength: *mut DWORD) -> BOOL;

    pub fn CapabilitiesRequestAndCapabilitiesReply(
        hMonitor: HANDLE,
        pszASCIICapabilitiesString: *mut c_char,
        dwCapabilitiesStringLengthInCharacters: DWORD,
    ) -> BOOL;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    pub fn GetLastError() -> DWORD;
}

#[cfg(not(windows))]
pub unsafe fn GetLastError() -> DWORD { 0 }

#[cfg(not(windows))]
pub unsafe fn EnumDisplayMonitors(
    _hdc: HDC,
    _lprcClip: *const RECT,
    _lpfnEnum: MONITORENUMPROC,
    _dwData: LPARAM,
) -> BOOL { 0 }

#[cfg(not(windows))]
pub unsafe fn GetNumberOfPhysicalMonitorsFromHMONITOR(
    _hMonitor: HMONITOR,
    _pdwNumberOfPhysicalMonitors: *mut DWORD,
) -> BOOL { 0 }

#[cfg(not(windows))]
pub unsafe fn GetPhysicalMonitorsFromHMONITOR(
    _hMonitor: HMONITOR,
    _dwPhysicalMonitorArraySize: DWORD,
    _pPhysicalMonitorArray: *mut PHYSICAL_MONITOR,
) -> BOOL { 0 }

#[cfg(not(windows))]
pub unsafe fn DestroyPhysicalMonitors(
    _dwPhysicalMonitorArraySize: DWORD,
    _pPhysicalMonitorArray: *mut PHYSICAL_MONITOR,
) -> BOOL { 0 }

#[cfg(not(windows))]
pub unsafe fn SetVCPFeature(_hMonitor: HANDLE, _bVCPCode: u8, _dwNewValue: DWORD) -> BOOL { 0 }

#[cfg(not(windows))]
pub unsafe fn GetVCPFeatureAndVCPFeatureReply(
    _hMonitor: HANDLE,
    _bVCPCode: u8,
    _pvct: *mut DWORD,
    _pvcp: *mut DWORD,
) -> BOOL { 0 }

#[cfg(not(windows))]
pub unsafe fn GetCapabilitiesStringLength(_hMonitor: HANDLE, _pdwCapabilitiesStringLength: *mut DWORD) -> BOOL { 0 }

#[cfg(not(windows))]
pub unsafe fn CapabilitiesRequestAndCapabilitiesReply(
    _hMonitor: HANDLE,
    _pszASCIICapabilitiesString: *mut c_char,
    _dwCapabilitiesStringLengthInCharacters: DWORD,
) -> BOOL { 0 }

pub fn parse_u32(s: &str) -> Result<u32, String> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex number '{s}': {e}"))
    } else {
        s.parse::<u32>().map_err(|e| format!("Invalid number '{s}': {e}"))
    }
}
