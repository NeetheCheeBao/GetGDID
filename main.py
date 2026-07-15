import base64
import os
import sys
import tempfile
import traceback
import webbrowser

try:
    import tkinter as tk
    from tkinter import messagebox
except Exception as e:
    sys.stderr.write("无法导入 tkinter: %s\n" % e)
    try:
        input("按回车退出...")
    except Exception:
        pass
    sys.exit(1)

if sys.platform != "win32":
    _r = tk.Tk()
    _r.withdraw()
    messagebox.showerror("错误", "本程序只能在 Windows 上运行。")
    _r.destroy()
    sys.exit(1)

import winreg

try:
    from ctypes import windll

    windll.shcore.SetProcessDpiAwareness(1)
except Exception:
    try:
        from ctypes import windll

        windll.user32.SetProcessDPIAware()
    except Exception:
        pass


REG_PATH = r"SOFTWARE\Microsoft\IdentityCRL\ExtendedProperties"
REG_VALUE = "LID"
GITHUB_URL = "https://github.com/NeetheCheeBao/GetGDID"
ICON_FILE = "icon0.png"

ICON_PNG_B64 = (
    "iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAACXBIWXMAAA7EAAAOxAGVKw4bAAAB"
    "2ElEQVRIS7XVz4tOURzH8ReeGGmYKSOa/EgyKT9mlJRGWNIks7Pgb1Ds7K3YWWo2/gPJZkopUpSw"
    "kdKosZjyoymMaBpdi3Of+5znnMu9s/CuU/d+zuf7Pffe8z3fy39mTSokbMcFTGE/dpT6At7iPu7h"
    "c6m3Zgg38QtFw/iBG9isJeP4IE/UNN7joAZOYUke3HZ8x6S/sBdf9MzL0XXTiL2fsFtJvMmP9a8+"
    "iTlcwXm8EjazwFYcETb5NvYJ8V0e4Ux0b0r+VN2KacMuefy52DBbYzgaGxo4Lo+f7U4Oyb/3PAa6"
    "hhYMCDFxjmVsIbxKuvo1q+eqPM/ZDkZjV8nLVGjB61TAaAcjqaq5hbRlpCMcrJQJPEzFBuqKYgmm"
    "5d/uHTb2fI1sEs5MmmcattVMFLiDdZpZj7vy+ELIDZ6UwoqwWb/L+6dCjdftSQen8UKeuBtbcbEU"
    "54QjPo7FUivwHBsqN4NC90yTxuNS5Rae8Fk58VVoE8fwRni7nT1rxWV50viB1vasgTF8FAwPkrk6"
    "RuWJC6GbjkW+Pg7ptewZnMBJoUrqWNGffFHosv9kQv5H2xMbIuIF5oWCaMUgbgl/qJ8Y7p+uWMA3"
    "XLe65lgxjMOpGHFAVOt1/AFJztEJi0m+ZQAAAABJRU5ErkJggg=="
)

WIN_W = 520
WIN_H = 260
WIN_MIN_W = 480
WIN_MIN_H = 240


def resource_path(name):
    if getattr(sys, "frozen", False):
        base = getattr(sys, "_MEIPASS", os.path.dirname(os.path.abspath(sys.executable)))
    else:
        base = os.path.dirname(os.path.abspath(__file__))
    return os.path.join(base, name)


def load_photoimage():
    path = resource_path(ICON_FILE)
    if os.path.isfile(path):
        try:
            return tk.PhotoImage(file=path)
        except Exception:
            pass
    try:
        return tk.PhotoImage(data=ICON_PNG_B64)
    except Exception:
        pass
    try:
        raw = base64.b64decode(ICON_PNG_B64)
        fd, tmp = tempfile.mkstemp(suffix=".png")
        os.close(fd)
        with open(tmp, "wb") as f:
            f.write(raw)
        try:
            return tk.PhotoImage(file=tmp)
        finally:
            try:
                os.remove(tmp)
            except Exception:
                pass
    except Exception:
        return None
    return None


def _lid_to_hex(lid):
    if lid is None:
        return None
    if isinstance(lid, bytes):
        if not lid:
            return None
        return lid.hex().upper()
    if isinstance(lid, int):
        return format(lid, "X")
    raw = str(lid).strip()
    if not raw:
        return None
    if raw.lower().startswith("0x"):
        raw = raw[2:]
    raw = raw.replace(" ", "").replace("-", "")
    return raw


def read_lid():
    try:
        key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, REG_PATH)
    except OSError:
        return None, None, (
            "未找到 IdentityCRL 注册表项。\n"
            "此设备可能尚未关联 Microsoft 账户。"
        )

    try:
        try:
            lid, _regtype = winreg.QueryValueEx(key, REG_VALUE)
        except OSError:
            return None, None, "已找到注册表项，但缺少 LID 值。"
    finally:
        winreg.CloseKey(key)

    raw = _lid_to_hex(lid)
    if not raw:
        return None, None, "已找到注册表项，但缺少 LID 值。"

    try:
        num = int(raw, 16)
    except ValueError:
        return None, None, "LID 不是有效的十六进制值：\n" + str(lid)

    return raw, "g:%d" % num, None


def show_error(title, text):
    try:
        r = tk.Tk()
        r.withdraw()
        messagebox.showerror(title, text)
        r.destroy()
    except Exception:
        try:
            sys.stderr.write("%s\n%s\n" % (title, text))
        except Exception:
            pass


class App(object):
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("获取 GDID")
        self.root.geometry("%dx%d" % (WIN_W, WIN_H))
        self.root.minsize(WIN_MIN_W, WIN_MIN_H)
        self.root.resizable(True, True)

        self.raw_hex = ""
        self.ms_fmt = ""
        self._github_img = None

        self._set_window_icon()
        self._build()
        self._center()
        self.root.bind("<Escape>", lambda e: self.root.destroy())
        self.root.bind("<Configure>", self._on_resize)
        self.refresh()

    def _set_window_icon(self):
        self._github_img = load_photoimage()
        if self._github_img is not None:
            try:
                self.root.iconphoto(True, self._github_img)
            except Exception:
                pass

    def _center(self):
        self.root.update_idletasks()
        w = self.root.winfo_width()
        h = self.root.winfo_height()
        if w <= 1:
            w = WIN_W
        if h <= 1:
            h = WIN_H
        x = max(0, (self.root.winfo_screenwidth() - w) // 2)
        y = max(0, (self.root.winfo_screenheight() - h) // 2)
        self.root.geometry("%dx%d+%d+%d" % (w, h, x, y))

    def _on_resize(self, event):
        if event.widget is not self.root:
            return
        if hasattr(self, "status_lbl") and self.status_lbl.winfo_exists():
            wrap = max(200, event.width - 60)
            self.status_lbl.configure(wraplength=wrap)

    def open_github(self):
        try:
            webbrowser.open(GITHUB_URL)
        except Exception as exc:
            messagebox.showerror("无法打开链接", str(exc))

    def _github_button(self, parent):
        if self._github_img is None:
            self._github_img = load_photoimage()
        img = self._github_img

        if img is not None:
            size = max(img.width(), img.height(), 24)
            return tk.Button(
                parent,
                image=img,
                width=size,
                height=size,
                bd=1,
                relief=tk.RAISED,
                cursor="hand2",
                command=self.open_github,
                takefocus=0,
            )

        return tk.Button(
            parent,
            text="GitHub",
            font=("Segoe UI", 8),
            bd=1,
            relief=tk.RAISED,
            cursor="hand2",
            command=self.open_github,
            takefocus=0,
        )

    def _build(self):
        pad = {"padx": 14, "pady": 6}
        frm = tk.Frame(self.root)
        frm.pack(fill=tk.BOTH, expand=True, padx=12, pady=10)

        header = tk.Frame(frm)
        header.pack(fill=tk.X, **pad)
        tk.Label(
            header,
            text="全局设备标识符 (GDID)",
            font=("Microsoft YaHei UI", 13, "bold"),
            anchor="w",
        ).pack(side=tk.LEFT, fill=tk.X, expand=True)
        self._github_button(header).pack(side=tk.RIGHT)

        self.status = tk.StringVar(value="正在读取…")
        self.status_lbl = tk.Label(
            frm,
            textvariable=self.status,
            fg="#555555",
            font=("Microsoft YaHei UI", 9),
            anchor="w",
            justify=tk.LEFT,
            wraplength=WIN_W - 50,
        )
        self.status_lbl.pack(fill=tk.X, **pad)

        btns = tk.Frame(frm)
        btns.pack(fill=tk.X, side=tk.BOTTOM, **pad)
        tk.Button(btns, text="刷新", width=10, command=self.refresh).pack(side=tk.LEFT)
        tk.Button(btns, text="全部复制", width=10, command=self.copy_both).pack(
            side=tk.LEFT, padx=(8, 0)
        )
        tk.Button(
            btns, text="全部复制(源)", width=12, command=self.copy_both_source
        ).pack(side=tk.LEFT, padx=(8, 0))
        tk.Button(btns, text="退出", width=10, command=self.root.destroy).pack(
            side=tk.RIGHT
        )

        mid = tk.Frame(frm)
        mid.pack(fill=tk.BOTH, expand=True)

        row1 = tk.Frame(mid)
        row1.pack(fill=tk.X, **pad)
        tk.Label(row1, text="原始十六进制 (LID)：", width=18, anchor="w").pack(
            side=tk.LEFT
        )
        self.hex_var = tk.StringVar()
        tk.Entry(
            row1,
            textvariable=self.hex_var,
            state="readonly",
            font=("Consolas", 11),
            readonlybackground="white",
        ).pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 8))
        tk.Button(row1, text="复制", width=8, command=self.copy_hex).pack(side=tk.RIGHT)

        row2 = tk.Frame(mid)
        row2.pack(fill=tk.X, **pad)
        tk.Label(row2, text="Microsoft 格式：", width=18, anchor="w").pack(side=tk.LEFT)
        self.ms_var = tk.StringVar()
        tk.Entry(
            row2,
            textvariable=self.ms_var,
            state="readonly",
            font=("Consolas", 11),
            readonlybackground="white",
        ).pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 8))
        tk.Button(row2, text="复制", width=8, command=self.copy_ms).pack(side=tk.RIGHT)

    def refresh(self):
        try:
            raw, ms, err = read_lid()
        except Exception as exc:
            raw, ms, err = None, None, "读取失败：\n%s" % exc

        if err:
            self.raw_hex = ""
            self.ms_fmt = ""
            self.hex_var.set("")
            self.ms_var.set("")
            self.status.set(err)
            self.status_lbl.config(fg="#b00020")
            return

        self.raw_hex = raw or ""
        self.ms_fmt = ms or ""
        self.hex_var.set(self.raw_hex)
        self.ms_var.set(self.ms_fmt)
        self.status.set("GDID 读取成功。")
        self.status_lbl.config(fg="#0a7a2f")

    def _copy(self, text, name):
        if not text:
            messagebox.showwarning("提示", "%s 为空，无法复制。" % name)
            return
        try:
            self.root.clipboard_clear()
            self.root.clipboard_append(text)
            self.root.update()
        except tk.TclError as exc:
            messagebox.showerror("复制失败", "无法写入剪贴板：\n%s" % exc)
            return
        self.status.set("已复制「%s」到剪贴板。" % name)
        self.status_lbl.config(fg="#1565c0")

    def copy_hex(self):
        self._copy(self.raw_hex, "原始十六进制 (LID)")

    def copy_ms(self):
        self._copy(self.ms_fmt, "Microsoft 格式")

    def copy_both(self):
        if not self.raw_hex and not self.ms_fmt:
            messagebox.showwarning("提示", "当前没有可用的 GDID 值。")
            return
        text = "原始十六进制 (LID)：%s\nMicrosoft 格式：%s" % (
            self.raw_hex,
            self.ms_fmt,
        )
        self._copy(text, "全部内容")

    def copy_both_source(self):
        if not self.raw_hex and not self.ms_fmt:
            messagebox.showwarning("提示", "当前没有可用的 GDID 值。")
            return
        text = (
            "=== Global Device Identifier (GDID) ===\n"
            "Raw Hex (LID): %s\n"
            "Microsoft Format (g:<decimal>): %s"
        ) % (self.raw_hex, self.ms_fmt)
        self._copy(text, "全部复制(源)")

    def run(self):
        self.root.mainloop()


def main():
    app = App()
    app.run()


if __name__ == "__main__":
    try:
        main()
    except Exception:
        err = traceback.format_exc()
        show_error("程序异常", err)
        try:
            if sys.stdin is not None and sys.stdin.isatty():
                input("按回车退出...")
        except Exception:
            pass
        sys.exit(1)
