# Zalo Portable — Multi-Profile Manager 💬

[🇻🇳 Tiếng Việt](#-tiếng-việt-vietnamese) | [🇬🇧 English](#-english)

A lightweight, portable desktop application built with **Tauri v2** that allows you to manage multiple isolated Zalo Web profiles simultaneously, just like Google Chrome profiles.

![Profile Manager](https://img.shields.io/badge/Tauri-v2-24C8DB?style=flat-square&logo=tauri)
![Rust](https://img.shields.io/badge/Rust-Backend-f74c00?style=flat-square&logo=rust)
![Vanilla JS](https://img.shields.io/badge/Frontend-Vanilla_JS-F7DF1E?style=flat-square&logo=javascript)

---

## 🇻🇳 Tiếng Việt (Vietnamese)

### ✨ Tính năng nổi bật

*   **🌐 Quản lý nhiều Profile**: Tạo không giới hạn số lượng profile Zalo. Mỗi profile hoạt động độc lập với một phiên đăng nhập (session) riêng biệt.
*   **💼 Hoàn toàn Portable (100% Portable)**: Mọi dữ liệu (Cookies, Local Storage, Session) được lưu trữ ngay trong thư mục `ProfileData/` nằm cùng cấp với file thực thi `.exe`. Bạn có thể copy toàn bộ thư mục app sang USB hoặc máy tính khác để dùng tiếp mà không mất dữ liệu đăng nhập.
*   **🎨 Giao diện Hiện đại (Glassmorphism UI)**: Quản lý profile với giao diện Dark Mode đẹp mắt, hỗ trợ đặt tên, chọn màu sắc nhận diện (Avatar) cho từng profile.
*   **⚡ Siêu nhẹ & Nhanh**: Viết bằng **Rust** và **Tauri v2**, sử dụng bộ nhớ cực thấp so với các ứng dụng bọc bằng Electron. Tối ưu WebView2 trên Windows.
*   **🔒 Cô lập dữ liệu**: Sử dụng Custom `data_directory` của WebView2 để đảm bảo các tài khoản không bao giờ bị xung đột với nhau.

### 🚀 Hướng dẫn cài đặt & Chạy thử

**Yêu cầu hệ thống:**
*   Hệ điều hành: Windows 10 / 11.
*   Môi trường Build: Node.js (v20+), Rust, và Visual Studio Build Tools (C++).

**Chạy ở chế độ Phát triển (Dev Mode):**
```bash
npm install
npm run tauri dev
```

**Đóng gói Ứng dụng (Build Production):**
Để đóng gói thành file `.exe` độc lập để sử dụng hàng ngày:
```bash
npm run tauri build
```
File thực thi sau khi build sẽ nằm tại: `src-tauri/target/release/zalo-portable.exe`.

---

## 🇬🇧 English

### ✨ Features

*   **🌐 Multi-Profile Management**: Create an unlimited number of Zalo profiles. Each profile operates independently with its own login session.
*   **💼 100% Portable**: All data (Cookies, Local Storage, Sessions) is stored directly in the `ProfileData/` folder located next to the `.exe` executable. You can copy the entire app folder to a USB drive or another computer to continue using it without losing login data.
*   **🎨 Modern Glassmorphism UI**: Manage profiles with a beautiful Dark Mode interface. Supports custom naming and color selection (Avatar) for easy identification of each profile.
*   **⚡ Ultra Lightweight & Fast**: Built with **Rust** and **Tauri v2**, consuming significantly less memory compared to Electron wrappers. Highly optimized for WebView2 on Windows.
*   **🔒 Data Isolation**: Utilizes custom WebView2 `data_directory` to ensure that accounts never conflict or share data with each other.

### 🚀 Quick Start & Installation

**System Requirements:**
*   OS: Windows 10 / 11.
*   Build Environment: Node.js (v20+), Rust, and Visual Studio Build Tools (C++).

**Run in Development Mode:**
```bash
npm install
npm run tauri dev
```

**Build for Production:**
To package the app into a standalone `.exe` file for daily use:
```bash
npm run tauri build
```
The compiled executable will be located at: `src-tauri/target/release/zalo-portable.exe`.

---

## 📂 App Data Structure / Cấu trúc thư mục dữ liệu

When the app runs, it automatically generates a local data folder next to the `.exe` file:
*(Khi chạy app, nó sẽ tự động tạo thư mục dữ liệu bên cạnh file `.exe`:)*

```text
/App-Directory/
 ├── zalo-portable.exe
 ├── profiles.json          <-- Profile configurations (names, colors)
 └── ProfileData/           <-- WebView runtime data (Cookies, LocalStorage)
      ├── profile_17000.../ <-- Browsing data for Profile 1
      ├── profile_17001.../ <-- Browsing data for Profile 2
      └── ...
```

## ⚠️ Important Note / Lưu ý
* Ensure your Windows PC has the [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) installed (usually pre-installed on Windows 10/11).
* **DO NOT** delete the `ProfileData` folder or `profiles.json` unless you want to log out of all accounts and reset the app.


