use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use super::windows::{KeyCode, KeySendMode, KeyboardTrait};

pub trait SendInputApi {
    fn send_input(&self, input_list: &[INPUT]) -> u32;
}

struct SendInputApiImpl {}
impl Default for SendInputApiImpl {
    fn default() -> Self {
        SendInputApiImpl {}
    }
}
impl SendInputApi for SendInputApiImpl {
    fn send_input(&self, input_list: &[INPUT]) -> u32 {
        unsafe { SendInput(input_list, std::mem::size_of::<INPUT>() as i32) }
    }
}

struct SendInputApiDelayedImpl {
    delay_millis: u64,
}
impl SendInputApiDelayedImpl {
    fn new(delay_millis: u64) -> Self {
        SendInputApiDelayedImpl {
            delay_millis: delay_millis,
        }
    }
}
impl SendInputApi for SendInputApiDelayedImpl {
    fn send_input(&self, input_list: &[INPUT]) -> u32 {
        for input in input_list{
            unsafe { SendInput(&[*input], std::mem::size_of::<INPUT>() as i32) };
            std::thread::sleep(Duration::from_millis(self.delay_millis));
        }
        0
    }
}

pub struct KeyboardImpl {
    pub keycode_chain: Vec<KeyCode>,
    pub sender: Box<dyn SendInputApi>,
}

impl KeyboardTrait for KeyboardImpl {
    fn send_key(&mut self) {
        let mut input_list = Vec::new();
        for keycode in self.keycode_chain.iter() {
            // keycode
            let flags_list = match keycode.key_send_mode() {
                KeySendMode::Immediate => {
                    vec![0, KEYEVENTF_KEYUP.0]
                }
                KeySendMode::KeyDown => {
                    vec![0]
                }
                KeySendMode::KeyUp => {
                    vec![KEYEVENTF_KEYUP.0]
                }
            };
            for flags in flags_list {
                let input = keyinput_generator_detail(
                    keycode.vk(),
                    keycode.scan_code(),
                    KEYBD_EVENT_FLAGS(keycode.flags | flags),
                );
                input_list.push(input);
            }
        }
        self.sender.send_input(&input_list);
    }
    fn append_input_chain(&mut self, key_code: KeyCode) {
        self.keycode_chain.push(key_code)
    }
}

impl KeyboardImpl {
    pub fn new_impl() -> Self {
        KeyboardImpl {
            keycode_chain: Vec::new(),
            sender: Box::new(SendInputApiImpl::default()),
        }
    }
    pub fn new_delay_impl(delay_millis: u64) -> Self {
        KeyboardImpl {
            keycode_chain: Vec::new(),
            sender: Box::new(SendInputApiDelayedImpl::new(delay_millis)),
        }
    }
}

pub fn keyinput_generator_detail(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    let mut kbd = KEYBDINPUT::default();
    let vk = vk;
    kbd.wVk = vk;
    kbd.wScan = scan;
    kbd.dwFlags = flags;
    kbd.time = 0;
    // ExtraInfoは特に意味のある値ではない。
    // このアプリから生成されたことを主張するだけの数値
    // KBDLLHOOKSTRUCT.dw_flagsでも判別は可能であるのであくまでもデバッグ支援用の値となる
    kbd.dwExtraInfo = 12345;

    let mut input = INPUT::default();
    input.r#type = INPUT_KEYBOARD;
    input.Anonymous.ki = kbd;
    input
}
