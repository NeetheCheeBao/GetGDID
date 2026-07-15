# GetGDID

[![Python](https://img.shields.io/badge/Python-3.x-blue.svg)](#)
[![Platform](https://img.shields.io/badge/Platform-Windows-win.svg)](#)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

一键读取 **Windows 全局设备标识符（GDID / LID）**

## 📸 工具截图

![img](/screenshot/demo1.png)

## ✨ 功能说明

| 功能 | 说明 |
| --- | --- |
| 读取 LID | 显示注册表中的原始十六进制 LID |
| Microsoft 格式 | 转换为 `g:<decimal>` 格式 |

## ⚙️ 原理说明

从注册表读取 `LID` 键值，再将十六进制值转换为十进制 GDID 格式。

* **路径**：`HKEY_CURRENT_USER\SOFTWARE\Microsoft\IdentityCRL\ExtendedProperties`
* **值名**：`LID`

## ⬇️ 下载使用

前往 [Releases](https://github.com/NeetheCheeBao/GetGDID/releases) 页面下载

## ⚖️ 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。
