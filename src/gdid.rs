use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_BINARY,
    REG_DWORD, REG_QWORD,
};

use crate::consts::{ERR_NO_KEY, ERR_NO_VALUE, REG_PATH, REG_VALUE};
use crate::win32::wide;

pub(crate) fn read_lid() -> Result<(String, String), String> {
    unsafe {
        let path = wide(REG_PATH);
        let mut hkey: HKEY = null_mut();
        if RegOpenKeyExW(HKEY_CURRENT_USER, path.as_ptr(), 0, KEY_READ, &mut hkey) != ERROR_SUCCESS
        {
            return Err(ERR_NO_KEY.to_string());
        }

        let name = wide(REG_VALUE);
        let mut kind: u32 = 0;
        let mut size: u32 = 0;
        let mut status =
            RegQueryValueExW(hkey, name.as_ptr(), null(), &mut kind, null_mut(), &mut size);
        if status == ERROR_SUCCESS && size > 0 {
            let mut buf = vec![0u8; size as usize];
            status = RegQueryValueExW(
                hkey,
                name.as_ptr(),
                null(),
                &mut kind,
                buf.as_mut_ptr(),
                &mut size,
            );
            if status == ERROR_SUCCESS {
                buf.truncate(size as usize);
                RegCloseKey(hkey);
                return decode_lid(kind, &buf);
            }
        }
        RegCloseKey(hkey);
        Err(ERR_NO_VALUE.to_string())
    }
}

fn decode_lid(kind: u32, buf: &[u8]) -> Result<(String, String), String> {
    let (candidate, display) = match kind {
        REG_BINARY => {
            if buf.is_empty() {
                return Err(ERR_NO_VALUE.to_string());
            }
            let hex = hex_upper(buf);
            (hex.clone(), hex)
        }
        REG_DWORD => {
            if buf.len() < 4 {
                return Err(ERR_NO_VALUE.to_string());
            }
            let v = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
            (format!("{:X}", v), v.to_string())
        }
        REG_QWORD => {
            if buf.len() < 8 {
                return Err(ERR_NO_VALUE.to_string());
            }
            let mut raw = [0u8; 8];
            raw.copy_from_slice(&buf[..8]);
            let v = u64::from_le_bytes(raw);
            (format!("{:X}", v), v.to_string())
        }
        _ => {
            let text = utf16_to_string(buf);
            if text.trim().is_empty() {
                return Err(ERR_NO_VALUE.to_string());
            }
            (text.clone(), text)
        }
    };

    let hex = normalize_hex(&candidate)
        .ok_or_else(|| format!("LID 不是有效的十六进制值：\n{}", display.trim()))?;
    let decimal = hex_to_decimal(&hex)
        .ok_or_else(|| format!("LID 不是有效的十六进制值：\n{}", display.trim()))?;
    Ok((hex, format!("g:{}", decimal)))
}

fn hex_upper(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 0x0F) as usize] as char);
    }
    out
}

// 手写十进制展开，LID 长度不受 u64 限制
fn hex_to_decimal(hex: &str) -> Option<String> {
    let mut digits: Vec<u8> = vec![0];
    for ch in hex.chars() {
        let mut carry = ch.to_digit(16)? as u32;
        for d in digits.iter_mut() {
            let cur = (*d as u32) * 16 + carry;
            *d = (cur % 10) as u8;
            carry = cur / 10;
        }
        while carry > 0 {
            digits.push((carry % 10) as u8);
            carry /= 10;
        }
    }
    while digits.len() > 1 && *digits.last().unwrap() == 0 {
        digits.pop();
    }
    Some(digits.iter().rev().map(|d| (b'0' + d) as char).collect())
}

fn normalize_hex(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let stripped = match trimmed.get(..2) {
        Some(prefix) if prefix.eq_ignore_ascii_case("0x") => &trimmed[2..],
        _ => trimmed,
    };
    let cleaned: String = stripped.chars().filter(|c| *c != ' ' && *c != '-').collect();
    if cleaned.is_empty() || !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(cleaned)
}

fn utf16_to_string(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|u| *u != 0)
        .collect();
    String::from_utf16_lossy(&units)
}
