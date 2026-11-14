use windows_sys::Win32::Foundation::COLORREF;

pub(crate) const APP_NAME: &str = "GetGDID";
pub(crate) const DEFAULT_VERSION: &str = "1.0.0";
pub(crate) const HEADER_TEXT: &str = "全局设备标识符 (GDID)";
pub(crate) const LABEL_HEX: &str = "原始十六进制 (LID)：";
pub(crate) const LABEL_MS: &str = "Microsoft 格式：";

pub(crate) const REG_PATH: &str = r"SOFTWARE\Microsoft\IdentityCRL\ExtendedProperties";
pub(crate) const REG_VALUE: &str = "LID";
pub(crate) const GITHUB_URL: &str = "https://github.com/NeetheCheeBao/GetGDID";

pub(crate) const ICON_ICO: &[u8] = include_bytes!("../assets/icon.ico");

pub(crate) const ID_REFRESH: u16 = 1001;
pub(crate) const ID_COPY_BOTH: u16 = 1002;
pub(crate) const ID_COPY_SRC: u16 = 1003;
pub(crate) const ID_EXIT: u16 = 1004;
pub(crate) const ID_COPY_HEX: u16 = 1005;
pub(crate) const ID_COPY_MS: u16 = 1006;
pub(crate) const ID_HEADER: u16 = 1007;

pub(crate) const SS_LEFT: u32 = 0x0000;
pub(crate) const SS_CENTERIMAGE: u32 = 0x0200;
pub(crate) const SS_NOTIFY: u32 = 0x0100;
pub(crate) const ES_READONLY: u32 = 0x0800;
pub(crate) const ES_AUTOHSCROLL: u32 = 0x0080;
pub(crate) const BS_PUSHBUTTON: u32 = 0x0000;
pub(crate) const VK_ESCAPE: u16 = 0x1B;
pub(crate) const CF_UNICODETEXT: u32 = 13;
pub(crate) const BK_TRANSPARENT: i32 = 1;
pub(crate) const ICON_SMALL_INDEX: usize = 0;
pub(crate) const ICON_BIG_INDEX: usize = 1;

// COLORREF 为 0x00BBGGRR，与常见的 #RRGGBB 相反
pub(crate) const CLR_STATUS: COLORREF = 0x0055_5555;
pub(crate) const CLR_ERROR: COLORREF = 0x0020_00B0;
pub(crate) const CLR_SUCCESS: COLORREF = 0x002F_7A0A;
pub(crate) const CLR_INFO: COLORREF = 0x00C0_6515;

pub(crate) const ERR_NO_KEY: &str =
    "未找到 IdentityCRL 注册表项。\n此设备可能尚未关联 Microsoft 账户。";
pub(crate) const ERR_NO_VALUE: &str = "已找到注册表项，但缺少 LID 值。";
