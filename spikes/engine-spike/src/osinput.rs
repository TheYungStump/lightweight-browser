//! Real OS-level input (SendInput) for tests that need genuine user input:
//! fullscreen activation and accelerators while a page has focus.

use std::mem::size_of;

use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT, SendInput, VIRTUAL_KEY, VK_CONTROL,
};
use windows::Win32::UI::WindowsAndMessaging::{SetCursorPos, SetForegroundWindow};

pub fn foreground(hwnd: HWND) {
    // SAFETY: plain Win32 call on our own top-level window.
    unsafe {
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Left-click at client coordinates (physical px) of `hwnd`.
pub fn click_client(hwnd: HWND, x: i32, y: i32) {
    let mut pt = POINT { x, y };
    // SAFETY: Win32 calls with valid pointers to stack values.
    unsafe {
        let _ = ClientToScreen(hwnd, &mut pt);
        let _ = SetCursorPos(pt.x, pt.y);
        let mk = |flags| INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: flags,
                    ..Default::default()
                },
            },
        };
        let inputs = [mk(MOUSEEVENTF_LEFTDOWN), mk(MOUSEEVENTF_LEFTUP)];
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

/// Press Ctrl+<letter>.
pub fn ctrl_key(letter: u8) {
    let key = |vk: VIRTUAL_KEY, up: bool| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                ..Default::default()
            },
        },
    };
    let k = VIRTUAL_KEY(letter.to_ascii_uppercase() as u16);
    let inputs = [
        key(VK_CONTROL, false),
        key(k, false),
        key(k, true),
        key(VK_CONTROL, true),
    ];
    // SAFETY: valid INPUT array of the stated element size.
    unsafe {
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}
