use crate::keyboard::windows_impl::*;
use windows::Win32::UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::MAPVK_VK_TO_VSC};

// Keyboard構造体やその実装は
// 外部公開するだけのためのI/Fにすぎないので、innerを呼び出すだけ。
pub struct Keyboard {
    inner: KeyboardImpl,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            inner: KeyboardImpl::new_impl(),
        }
    }
    pub fn new_delay(&mut self, delay_millis: u64) -> &Self {
        if delay_millis > 0 {
            self.inner.sender = KeyboardImpl::new_delay_impl(delay_millis).sender;
        }
        self
    }
}
impl KeyboardTrait for Keyboard {
    fn send_key(&mut self) {
        self.inner.send_key()
    }
    fn append_input_chain(&mut self, key_code: KeyCode) {
        self.inner.append_input_chain(key_code)
    }
    fn clear_input_chain(&mut self) {
        self.inner.keycode_chain.clear()
    }
}

pub trait KeyboardTrait {
    fn send_key(&mut self);
    fn append_input_chain(&mut self, key_code: KeyCode);
    fn clear_input_chain(&mut self);
}

#[derive(Debug, Clone)]
pub struct KeyCode {
    vk: VIRTUAL_KEY,
    scan_code: u16,
    flags: u32,
    key_send_mode: KeySendMode,
    extra_info: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KeySendMode {
    Immediate, // WM_KEYUP + WM_KEYDOWN
    KeyUp,     // WM_KEYUP
    KeyDown,   // WM_KEYDOWN
}

impl KeyCode {
    pub fn vk(&self) -> VIRTUAL_KEY {
        self.vk
    }
    pub fn scan_code(&self) -> u16 {
        self.scan_code
    }
    pub fn key_send_mode(&self) -> KeySendMode {
        self.key_send_mode
    }
    pub fn extra_info(&self)->usize{
        self.extra_info
    }
    pub fn flags(&self)->u32{
        self.flags
    }
}

impl Default for KeyCode {
    fn default() -> Self {
        KeyCode {
            vk: VIRTUAL_KEY(0),
            scan_code: 0,
            flags: 0,
            key_send_mode: KeySendMode::Immediate,
            extra_info:12345 // 物理キー入力は0なのでそれとは違う値なら何でもいい。
        }
    }
}

// OS固有の実装になる。
// I/Fも別
#[derive(Debug)]
pub struct KeycodeBuilder {
    data: Vec<KeyCode>,
    key_code: KeyCode,
}
impl Default for KeycodeBuilder {
    fn default() -> Self {
        KeycodeBuilder {
            data: Vec::new(),
            key_code: KeyCode::default(),
        }
    }
}

trait ShiftKeyAction {
    fn action(&mut self, v: &mut Vec<KeyCode>);
}

struct ShiftIgnore;
impl ShiftIgnore {
    fn new() -> Box<Self> {
        Box::new(ShiftIgnore {})
    }
}
impl ShiftKeyAction for ShiftIgnore {
    fn action(&mut self, _v: &mut Vec<KeyCode>) {}
}
struct Shift {
    state: bool,
}
impl Shift {
    fn new() -> Box<Self> {
        Box::new(Shift { state: false })
    }
}
impl ShiftKeyAction for Shift {
    fn action(&mut self, v: &mut Vec<KeyCode>) {
        let kc = KeycodeBuilder::new()
            .vk(160)
            .scan_code(virtual_key_to_scancode(VIRTUAL_KEY(160)))
            .key_send_mode(if self.state {
                KeySendMode::KeyUp
            } else {
                KeySendMode::KeyDown
            })
            .build();
        self.state = !self.state;
        v.push(kc);
    }
}

fn get_proc(
    flag: bool,
    flag_true_act: Box<dyn ShiftKeyAction>,
    flag_false_act: Box<dyn ShiftKeyAction>,
) -> Box<dyn ShiftKeyAction> {
    if flag {
        flag_true_act
    } else {
        flag_false_act
    }
}

use windows::Win32::Foundation::*;
impl KeycodeBuilder {
    fn new() -> Self {
        KeycodeBuilder::default()
    }
    /// 文字(char型)からキーストロークを生成する
    /// 本関数を実行した場合、vk/scan_code/key_send_modeで設定された内容はすべて無視される
    pub fn char_build(&mut self, char: char) -> Vec<KeyCode> {
        let mut utf16 = Vec::new();
        if char.is_ascii() {
            let (shift, _ctrl, vk) = unsafe {
                let kl = GetKeyboardLayout(0);
                // 文字 から VirtualKey へ変換する
                let vk = VIRTUAL_KEY(VkKeyScanExA(CHAR(char as u8), kl) as u16);
                ((vk.0 & 0x100) != 0, (vk.0 & 0x200) != 0, vk)
            };
            // shiftが押されているかどうか
            let mut shift_act = get_proc(shift, Shift::new(), ShiftIgnore::new());
            shift_act.action(&mut self.data);
            let mut kc = KeyCode::default();
            kc.vk = VIRTUAL_KEY(vk.0 & 0xff);
            // VirtualKeyからスキャンコードへ変換する
            kc.scan_code = virtual_key_to_scancode(vk);
            self.data.push(kc);
            shift_act.action(&mut self.data);
        } else {
            utf16.push(0);
            self.data.push(KeyCode::default());
            if char as u32 > 0xffff {
                utf16.push(0);
                self.data.push(KeyCode::default());
            }
            char.encode_utf16(&mut utf16);
            for (i, scan_code) in utf16.iter().enumerate() {
                self.data[i].flags = KEYEVENTF_UNICODE.0;
                self.data[i].scan_code = *scan_code;
            }
        };
        self.data.clone()
    }
    pub fn vk(&mut self, vk: u16) -> &mut Self {
        self.key_code.vk = VIRTUAL_KEY(vk);
        self
    }
    pub fn scan_code(&mut self, scan_code: u16) -> &mut Self {
        self.key_code.scan_code = scan_code;
        self
    }
    pub fn key_send_mode(&mut self, key_send_mode: KeySendMode) -> &mut Self {
        self.key_code.key_send_mode = key_send_mode;
        self
    }
    pub fn flags(&mut self, flags: u32) -> &mut Self {
        self.key_code.flags = flags;
        self
    }
    pub fn extra_info(&mut self,info:usize)->&mut Self{
        self.key_code.extra_info=info;
        self
    }
    pub fn build(&self) -> KeyCode {
        self.key_code.clone()
    }
}

pub fn virtual_key_to_scancode(vk: VIRTUAL_KEY) -> u16 {
    unsafe { MapVirtualKeyA(vk.0 as u32, MAPVK_VK_TO_VSC as u32) as u16 }
}
