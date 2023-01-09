use crate::keyboard::windows::*;
use crate::keyboard::{
    windows::{virtual_key_to_scancode, KeycodeBuilder},
    windows_impl::SendInputApi,
};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::windows_impl::*;

#[test]
fn keyboard_impl_test() {
    /// モックの実装
    struct SenderMock {}
    impl SendInputApi for SenderMock {
        fn send_input(
            &self,
            input_list: &[windows::Win32::UI::Input::KeyboardAndMouse::INPUT],
        ) -> u32 {
            for input in input_list {
                let ki = unsafe { input.Anonymous.ki };
                println!("{ki:?}");
            }
            0
        }
    }
    let mut keyboard_impl = KeyboardImpl {
        keycode_chain: Vec::new(),
        sender: Box::new(SenderMock {}),
    };
    for ch in ['A'] {
        KeycodeBuilder::default()
            .char_build(ch)
            .iter()
            .for_each(|key_code| keyboard_impl.append_input_chain(key_code.clone()));
    }
    keyboard_impl.send_key();
}

#[test]
fn keyboard_impl_test2() {
    /// モックの実装
    struct SenderMock {}
    impl SendInputApi for SenderMock {
        fn send_input(
            &self,
            input_list: &[windows::Win32::UI::Input::KeyboardAndMouse::INPUT],
        ) -> u32 {
            for input in input_list {
                let ki = unsafe { input.Anonymous.ki };
                let test_data = [
                    // (vk,scan,key_up,unicode,scancode)
                    (162, 29, false, false, false), // CTRL なのでwVkとscanが有効であり、その他フラグは全て零
                    (0, 'c' as u16, false, false, true),
                    (0, 'c' as u16, true, false, true),
                    (0, 'v' as u16, false, false, true),
                    (0, 'v' as u16, true, false, true),
                    (162, 29, true, false, false),
                    (0, 55356, false, true, false), // 🍣の1個め KeyDown
                    (0, 55356, true, true, false),  // 🍣の1個め KeyUp
                    (0, 57187, false, true, false), // 🍣の2個め KeyDown
                    (0, 57187, true, true, false),  // 🍣の2個め KeyUp
                ];
                assert_eq!(input_list.len(), test_data.len());
                for (input, test) in input_list.iter().zip(test_data) {
                    let kbd = unsafe { input.Anonymous.ki };
                    println!("{:?}", kbd);
                    let kbd = unsafe { input.Anonymous.ki };
                    assert_eq!(kbd.wVk.0, test.0);
                    assert_eq!(kbd.wScan, test.1);
                    assert_eq!((kbd.dwFlags.0 & KEYEVENTF_KEYUP.0) != 0, test.2);
                    assert_eq!((kbd.dwFlags.0 & KEYEVENTF_UNICODE.0) != 0, test.3);
                    assert_eq!((kbd.dwFlags.0 & KEYEVENTF_SCANCODE.0) != 0, test.4);
                }
            }
            0
        }
    }
    let mut keyboard_impl = KeyboardImpl {
        keycode_chain: Vec::new(),
        sender: Box::new(SenderMock {}),
    };
    // ^C-c ^C-v
    keyboard_impl.append_input_chain(
        KeycodeBuilder::default()
            .vk(VK_LCONTROL.0)
            .scan_code(virtual_key_to_scancode(VK_LCONTROL))
            .key_send_mode(KeySendMode::KeyDown)
            .build(),
    );
    for ch in ['c', 'v'] {
        KeycodeBuilder::default()
            .char_build(ch)
            .iter()
            .for_each(|key_code| keyboard_impl.append_input_chain(key_code.clone()));
    }

    keyboard_impl.append_input_chain(
        KeycodeBuilder::default()
            .vk(VK_LCONTROL.0)
            .scan_code(virtual_key_to_scancode(VK_LCONTROL))
            .key_send_mode(KeySendMode::KeyUp)
            .build(),
    );
    KeycodeBuilder::default()
        .char_build('🍣')
        .iter()
        .for_each(|key_code| {
            println!("register input: {:?}", key_code);
            keyboard_impl.append_input_chain(key_code.clone())
        });
    keyboard_impl.send_key();
}

#[test]
fn input_test() {
    let mut keyboard_impl = crate::keyboard::windows::Keyboard::new();
    for c in ['あ','🍣','a', 'A', '`', '@'] {
        KeycodeBuilder::default()
            .char_build(c)
            .iter()
            .for_each(|key_code| {
                println!("{:?}", key_code);
                keyboard_impl.append_input_chain(key_code.clone())
            });
    }
    keyboard_impl.send_key();
}
#[test]
fn input_test2() {
    let mut keyboard_impl = crate::keyboard::windows::Keyboard::new();
    for c in 0x21..0x7f {
        KeycodeBuilder::default()
            .char_build(char::from_u32(c).unwrap())
            .iter()
            .for_each(|key_code| {
                println!("{:?}", key_code);
                keyboard_impl.append_input_chain(key_code.clone())
            });
    }
    keyboard_impl.send_key();
}
