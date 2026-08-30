# Notify

## Wireless Android Notification Mirror for Desktop

Notify is a privacy-conscious desktop companion that mirrors Android notifications to a computer over the local network. It includes a Tauri desktop application and a native Android companion app.

The project is designed for people who want to read and manage phone notifications from their desktop without relying on a cloud relay or requiring Android Debug Bridge (ADB) for normal operation.

> **Status:** Early-stage project (`0.1.0`). The core pairing, local discovery, WebSocket synchronization, notification storage, desktop notifications, quick replies, clipboard transfer, and Android background service are implemented. Some packaging and production-hardening tasks may still be required for a release build.

---

## Contents

- [Features](#features)
- [How It Works](#how-it-works)
- [Architecture](#architecture)
- [Requirements](#requirements)
- [Quick Start](#quick-start)
- [Desktop Development](#desktop-development)
- [Android Companion Setup](#android-companion-setup)
- [Pairing and Connection](#pairing-and-connection)
- [Building the APK](#building-the-apk)
- [Building the Desktop Application](#building-the-desktop-application)
- [Configuration and Ports](#configuration-and-ports)
- [Privacy and Security](#privacy-and-security)
- [Troubleshooting](#troubleshooting)
- [Project Structure](#project-structure)
- [Development Notes](#development-notes)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [راهنمای فارسی](#راهنمای-فارسی)

---

## Features

### Desktop application

- Modern React and TypeScript interface.
- Tauri 2 desktop shell with a Rust backend.
- Local Android companion server.
- UDP device discovery on the local network.
- WebSocket connection for real-time synchronization.
- Desktop notification display.
- Notification history backed by SQLite.
- OTP and verification-code detection.
- Notification update deduplication to reduce duplicate alerts.
- Quick replies where the Android notification supports them.
- Clipboard synchronization from Android to desktop.
- Battery, storage, and connection telemetry.
- QR-code pairing data generation.
- Optional tray, autostart, shell, and opener integrations.

### Android companion

- Native Kotlin Android application.
- Dark mobile interface.
- QR-code scanning for pairing.
- Automatic PC discovery over Wi-Fi.
- Available-PC list: tap a discovered device to connect.
- Manual IP and port fallback.
- Android Notification Listener integration.
- Foreground background-sync service.
- Boot receiver for restarting companion behavior after device reboot.
- Battery-optimization guidance.
- Notification permission guidance for Android versions that require it.
- VPN-aware local-network discovery and connection probing.

---

## How It Works

1. The desktop application starts a local WebSocket server on port `27890`.
2. It also listens for UDP discovery requests on port `27891`.
3. The Android app broadcasts a discovery request on the current Wi-Fi network.
4. The desktop responds with its candidate LAN addresses and WebSocket port.
5. The Android app displays discovered devices in a list.
6. When a device is selected, the app probes the advertised addresses and connects to the reachable one.
7. Android sends notification events, removals, quick-reply metadata, clipboard data, and telemetry over WebSocket.
8. The desktop stores, displays, and processes those events locally.

The normal connection path does not require ADB. ADB-related code may still exist for optional device-control functionality, but notification mirroring through the companion is network based.

---

## Architecture

```text
+-----------------------+             Local Wi-Fi / LAN             +-----------------------+
| Android Companion     |                                             | Notify Desktop        |
|                       |                                             |                       |
| Notification Listener | -- UDP discovery :27891 -----------------> | UDP discovery server  |
| Kotlin WebSocket      | <---------------- WebSocket :27890 ------> | Rust WebSocket server |
| Background service    |                                             | SQLite + React UI     |
+-----------------------+                                             +-----------------------+
```

### Main technologies

| Area | Technology |
| --- | --- |
| Desktop UI | React 19, TypeScript, Vite |
| Desktop shell | Tauri 2 |
| Desktop backend | Rust, Tokio |
| Desktop storage | SQLite through `rusqlite` |
| Mobile app | Kotlin, Android SDK |
| Mobile networking | OkHttp WebSocket, Kotlin coroutines |
| Pairing | QR Code and UDP discovery |
| QR scanning | ZXing Android Embedded |
| Styling | Tailwind CSS on the desktop; programmatic native Android views in the companion |

---

## Requirements

### Desktop development

Install:

- Node.js 18+ recommended.
- npm (the repository includes `package-lock.json`).
- Rust stable toolchain and Cargo.
- Tauri system prerequisites for your operating system.
- A desktop environment supported by Tauri 2.

For official Tauri prerequisites, see:
<https://v2.tauri.app/start/prerequisites/>

### Android development

Install:

- Android Studio or Android SDK command-line tools.
- Android SDK Platform 36.
- Android Build Tools `36.0.0`.
- JDK 17.
- Gradle 8.13, or use the compatible Gradle installation configured for the project.
- An Android device running Android 7.0/API 24 or newer.

The Android module currently declares:

- `minSdk 24`
- `targetSdk 36`
- `compileSdk 36`

The Android device and desktop must normally be connected to the same local network. If the desktop is behind a VPN, virtual adapter, firewall, or multiple network interfaces, see [Troubleshooting](#troubleshooting).

---

## Quick Start

### 1. Clone the project

```bash
git clone <repository-url>
cd notify
```

### 2. Install JavaScript dependencies

```bash
npm install
```

### 3. Start the desktop development application

```bash
npm run tauri dev
```

This runs the Vite frontend and launches the Tauri desktop application.

If you only want to run the web frontend:

```bash
npm run dev
```

### 4. Build-check the desktop frontend

```bash
npm run build
```

### 5. Build and install the Android companion

Open the `android-companion` directory in Android Studio, configure the local Android SDK, and run the `app` configuration on a connected device.

Or build the debug APK using the Gradle command described below.

### 6. Pair the devices

1. Start Notify on the desktop.
2. Start the Android companion.
3. Make sure both devices are on the same Wi-Fi/LAN.
4. Tap **Auto-Discover PC on Wi-Fi**.
5. Tap a device in **Available devices**.
6. Alternatively, use **Scan PC QR Code** from the desktop pairing screen.
7. Grant Notification Access when prompted.
8. Keep the companion background service enabled for reliable synchronization.

---

## Desktop Development

### Frontend commands

```bash
npm run dev       # Start Vite development server
npm run build     # TypeScript check and production frontend build
npm run preview   # Preview the Vite production build
npm run tauri     # Invoke the Tauri CLI
npm run tauri dev # Run the desktop app in development mode
```

The desktop Tauri configuration uses:

- Development URL: `http://localhost:1420`
- Frontend output: `dist`
- Product name: `Notify`
- Application identifier: `com.notify.desktop`
- Default window: `940x660`

### Rust commands

From `src-tauri`:

```bash
cargo check
cargo test
cargo fmt --all
cargo clippy --all-targets --all-features
```

Some commands may require additional platform dependencies or may take time on the first run while Cargo downloads and compiles crates.

---

## Android Companion Setup

### Using Android Studio

1. Open the `android-companion` directory in Android Studio.
2. Confirm that Android Studio detects the Android application module.
3. Set the Android SDK location in `local.properties` if Android Studio does not detect it automatically.
4. Select a physical phone or emulator.
5. Run the `app` configuration.
6. On the phone, open Notify Companion.

### Required Android permissions

The app declares permissions for:

- Internet and network state.
- Foreground service and data synchronization.
- Posting notifications on newer Android versions.
- Camera access for QR scanning.
- Boot completion.
- Battery-optimization exemption requests.

The most important user-facing permission is **Notification Access**. Without it, Android will not allow the companion to read notification events.

Open the gear icon in the top-right of the mobile app to access the permission and battery settings.

### Background behavior

The Android companion uses a foreground service to keep the synchronization process alive. Some device manufacturers aggressively stop background applications. If synchronization stops when the screen is off:

- Open the app settings panel.
- Disable battery optimization for Notify Companion.
- Allow background activity/autostart if the device provides those controls.
- Avoid force-stopping the app.

---

## Pairing and Connection

### QR pairing

The desktop generates pairing data containing:

- Primary desktop LAN IP.
- Additional candidate LAN IPs.
- WebSocket port.
- Pairing secret token.
- Desktop display name.

The Android app scans the QR code, fills the connection information, stores the pairing data, and attempts the connection.

### Wi-Fi discovery

The Android companion sends `NOTIFY_DISCOVER` to UDP port `27891`. The desktop responds with `NOTIFY_SERVER` packets containing candidate addresses.

The mobile UI shows each candidate as an available device. Tapping one connects to that address directly. This avoids hiding multiple addresses in one input field and makes VPN/virtual-interface behavior easier to understand.

### Manual connection

If discovery is unavailable, use the **Manual connection** fields:

- Desktop IP address, for example `192.168.1.20`.
- WebSocket port, normally `27890`.

Then tap **Connect to PC**.

### Disconnecting

The mobile **Disconnect from PC** action stops the active connection and prevents automatic reconnection until the app is asked to connect again. The desktop also supports pausing the companion server.

---

## Building the APK

The project currently contains generated Gradle output and a local Gradle distribution may already be available on a development machine. Use the Gradle executable available in your environment.

### Debug APK

From `android-companion`:

```bash
gradle :app:assembleDebug
```

On Windows, if `gradle` is not on `PATH`, use the Gradle executable installed by Android Studio/Gradle locally, or open the project in Android Studio and run **Build > Make Project**.

The output is normally:

```text
android-companion/app/build/outputs/apk/debug/app-debug.apk
```

### Install the debug APK with Android Studio or ADB

If ADB is installed and the device is authorized:

```bash
adb install -r android-companion/app/build/outputs/apk/debug/app-debug.apk
```

If more than one device is connected:

```bash
adb devices
adb -s <device-serial> install -r android-companion/app/build/outputs/apk/debug/app-debug.apk
```

> ADB is used here only to install/debug the APK. Normal notification synchronization does not require ADB.

### Release APK

Before distributing a release APK, configure a release signing key and review:

- `applicationId`.
- Version code and version name.
- ProGuard/R8 settings.
- Network security requirements.
- Pairing-secret lifecycle.
- Android foreground-service behavior.

Then build:

```bash
gradle :app:assembleRelease
```

Do not distribute an unsigned or debug-signed APK as a production release.

---

## Building the Desktop Application

After installing the prerequisites:

```bash
npm install
npm run tauri build
```

Tauri places platform-specific bundles under the generated build directories. The exact installer format depends on the operating system and Tauri bundle configuration.

Before making a public release, verify:

- The Android companion can discover the packaged desktop app.
- Firewall rules allow TCP `27890` and UDP `27891` on the local network.
- The app has a stable data directory for SQLite.
- Tray/autostart behavior is intentional.
- Release signing and installer metadata are configured.

---

## Configuration and Ports

| Setting | Value | Purpose |
| --- | ---: | --- |
| Desktop WebSocket | `27890/TCP` | Android-to-desktop live connection |
| Desktop discovery | `27891/UDP` | Local-network device discovery |
| Desktop dev server | `1420/TCP` | Vite/Tauri development frontend |
| Android minimum SDK | `24` | Android 7.0 and newer |
| Android target SDK | `36` | Current target configured by the project |
| Desktop identifier | `com.notify.desktop` | Tauri application identifier |
| Android identifier | `com.notify.companion` | Android application ID |

The desktop server binds to all interfaces so the phone can reach it over the LAN. A host firewall may still block incoming connections.

---

## Privacy and Security

Notify is intended to keep notification data on the local network and on the paired devices.

Important considerations:

- Notification contents can include private messages, OTPs, emails, and personal information.
- The WebSocket server is intended for a trusted local network, not direct exposure to the public internet.
- Pairing data includes a secret token. Do not publish QR codes or pairing strings.
- Do not forward ports `27890` or `27891` from your router.
- Use a trusted Wi-Fi network.
- Review desktop notification permissions and Android Notification Access carefully.
- A future production release should enforce pairing-token validation on the server and use authenticated/encrypted transport where appropriate.

The project currently uses `ws://` for the local WebSocket connection. This is suitable only for a controlled local network. Do not treat it as internet-safe transport without adding authentication and encryption.

---

## Troubleshooting

### The phone cannot find the desktop

1. Confirm both devices are on the same Wi-Fi/LAN.
2. Confirm Notify is running on the desktop.
3. Check that UDP `27891` is allowed through the desktop firewall.
4. Check that TCP `27890` is allowed through the desktop firewall.
5. Disable client isolation/AP isolation on the Wi-Fi router.
6. Temporarily disconnect VPNs and test again.
7. Try the manual IP connection.
8. Check whether the desktop is connected through a guest network.

### The list contains strange or unreachable IP addresses

The desktop can have several interfaces, including VPN, Docker, VirtualBox, WSL, Hyper-V, and physical Wi-Fi/Ethernet adapters. Notify advertises candidate LAN addresses and the Android app probes them. Select the address that belongs to the same reachable network as the phone.

### The app connects but notifications do not arrive

1. Open Android system settings.
2. Search for **Notification access**.
3. Enable access for Notify Companion.
4. Reopen the companion.
5. Check that the desktop connection status is connected.
6. Send a new test notification.

### Synchronization stops in the background

Open the mobile gear/settings dialog and disable battery optimization. Also check manufacturer-specific background restrictions, autostart settings, and protected-app lists.

### QR scanning does not work

- Grant camera permission.
- Ensure the QR code is fully visible and not blurry.
- Increase screen brightness on the desktop.
- Use Wi-Fi discovery or manual IP connection as a fallback.

### Desktop notifications are missing

Check operating-system notification permissions for Notify and confirm that the desktop event pipeline is running. The notification history database may still contain received events even when native toast display is disabled.

### The build fails because Gradle is not found

Install Gradle or use Android Studio's configured Gradle environment. On Windows, Android Studio may have a local Gradle distribution under the user Gradle directory. The project does not currently include a committed `gradlew` wrapper script, so the command name may differ between machines.

### The build fails because Android SDK is missing

Set `sdk.dir` in `android-companion/local.properties`, for example on Windows:

```properties
sdk.dir=C\\:\\Users\\<username>\\AppData\\Local\\Android\\Sdk
```

Do not commit `local.properties`; it contains machine-specific paths.

---

## Project Structure

```text
notify/
├── android-companion/
│   └── app/src/main/
│       ├── java/com/notify/companion/
│       │   ├── network/       # WebSocket, discovery, protocol, telemetry
│       │   ├── service/       # Notification listener, foreground service, boot receiver
│       │   └── ui/            # Main Android activity and mobile UI
│       └── res/                # Android resources, theme, icons
├── public/                     # Static frontend assets
├── src/
│   ├── components/             # React UI components
│   ├── stores/                 # Client-side state stores
│   └── ...                     # Frontend application code
├── src-tauri/
│   ├── src/companion/           # UDP discovery and WebSocket server
│   ├── src/notifications/       # Notification parsing, storage, OTP detection
│   ├── src/adb/                 # Optional ADB integration
│   ├── src/controls/            # Desktop/device controls
│   ├── src/storage/             # SQLite database layer
│   ├── src/telemetry/           # Telemetry support
│   └── tauri.conf.json          # Desktop packaging and runtime configuration
├── index.html
├── package.json
├── package-lock.json
├── tsconfig.json
└── vite.config.ts
```

---

## Development Notes

### Keep generated files out of source changes

Build directories such as `android-companion/app/build`, Gradle caches, and local SDK configuration are machine-generated. They should generally not be reviewed as application source changes.

### UI changes

The Android companion currently builds its main screen programmatically in `MainActivity.kt`. The mobile interface uses a dark theme, rounded controls, available-device cards, and a modal Settings dialog.

The desktop interface is implemented in React/TypeScript and should be changed independently from the mobile app when the task is mobile-only.

### Network changes

When changing discovery or connection behavior, test all of these cases:

- One physical Wi-Fi interface.
- Ethernet plus Wi-Fi.
- VPN enabled.
- Multiple virtual adapters.
- No desktop candidate address.
- More than one desktop on the network.
- Desktop firewall enabled.
- Phone leaving and rejoining the Wi-Fi.

### Notification changes

When changing notification processing, test:

- New notification.
- Updated notification.
- Removed notification.
- OTP/verification notification.
- Notification with only body text.
- Notification from a work profile or parallel space.
- Reply-capable notification.
- Duplicate progress/update events.

---

## Roadmap

Potential future improvements:

- Commit Gradle wrapper scripts and document reproducible Android builds.
- Add automated Android UI and connection tests.
- Add authenticated pairing-token validation on the desktop server.
- Add optional TLS or another authenticated encrypted transport.
- Support multiple simultaneously connected Android devices.
- Improve desktop and Android connection diagnostics.
- Add network interface details to discovery cards without exposing noisy candidates.
- Add an explicit connection history and remembered-device management screen.
- Add configurable notification filters and per-app rules.
- Add richer quick-reply support and notification actions.
- Add signed release APK and desktop installers through CI.
- Add accessibility verification for both interfaces.

---

## Contributing

1. Create a focused branch.
2. Keep desktop and mobile changes separated when possible.
3. Follow the existing TypeScript, Rust, and Kotlin conventions.
4. Run the relevant checks before opening a pull request:

```bash
npm run build
cd src-tauri && cargo check
cd ../android-companion && gradle :app:assembleDebug
```

5. Describe network conditions and Android version when reporting connection bugs.
6. Never include notification contents, pairing secrets, APK signing keys, or local SDK paths in issues or commits.

---

## License

No license file is currently included in the repository. Until a license is added, all rights are reserved by the copyright holder. Add an explicit open-source license before redistributing the project or accepting external contributions under open-source terms.

---

# راهنمای فارسی

## معرفی پروژه

Notify یک برنامه دسکتاپ و همراه اندرویدی است که اعلان‌های گوشی را از طریق شبکه محلی به کامپیوتر منتقل می‌کند. این پروژه از دو بخش اصلی تشکیل شده است:

1. **برنامه دسکتاپ:** ساخته‌شده با Tauri، React، TypeScript و Rust.
2. **برنامه همراه اندروید:** ساخته‌شده با Kotlin و Android SDK.

هدف Notify این است که اعلان‌های اندروید را بدون نیاز به سرویس ابری و بدون نیاز به ADB در استفاده عادی، روی دسکتاپ نمایش دهد. ارتباط اصلی از طریق Wi-Fi یا LAN انجام می‌شود و داده‌های اعلان روی دستگاه‌های جفت‌شده باقی می‌مانند.

> **وضعیت پروژه:** نسخه فعلی `0.1.0` و در مرحله توسعه اولیه است. قابلیت‌های جفت‌سازی، کشف دستگاه، ارتباط WebSocket، ذخیره اعلان، نمایش اعلان دسکتاپ، پاسخ سریع، انتقال کلیپ‌بورد و سرویس پس‌زمینه اندروید پیاده‌سازی شده‌اند؛ اما برای انتشار عمومی هنوز به تست و سخت‌سازی بیشتری نیاز است.

---

## امکانات

### امکانات برنامه دسکتاپ

- رابط کاربری مدرن با React و TypeScript.
- پوسته دسکتاپ با Tauri 2.
- هسته Rust برای سرور ارتباطی.
- کشف گوشی از طریق UDP در شبکه محلی.
- ارتباط لحظه‌ای WebSocket.
- نمایش اعلان روی دسکتاپ.
- ذخیره تاریخچه اعلان‌ها در SQLite.
- تشخیص کدهای یک‌بارمصرف و تأیید هویت.
- جلوگیری از نمایش چندباره اعلان‌های تکراری.
- پشتیبانی از پاسخ سریع در اعلان‌های سازگار.
- انتقال کلیپ‌بورد از گوشی به دسکتاپ.
- ارسال اطلاعات باتری، حافظه و وضعیت اتصال.
- تولید QR برای جفت‌سازی.
- پشتیبانی از tray، اجرای خودکار، shell و opener از طریق افزونه‌های Tauri.

### امکانات برنامه اندروید

- برنامه native با Kotlin.
- رابط کاربری تیره.
- اسکن QR برای جفت‌سازی.
- پیدا کردن خودکار کامپیوتر در Wi-Fi.
- نمایش کامپیوترهای قابل دسترس به صورت فهرست؛ با لمس هر مورد اتصال انجام می‌شود.
- اتصال دستی با IP و Port.
- استفاده از Android Notification Listener.
- سرویس foreground برای ادامه همگام‌سازی در پس‌زمینه.
- راه‌اندازی مجدد سرویس پس از روشن شدن گوشی.
- راهنمای غیرفعال‌کردن بهینه‌سازی باتری.
- شناسایی بهتر شبکه واقعی هنگام فعال بودن VPN.

---

## معماری و روند کار

1. برنامه دسکتاپ یک سرور WebSocket روی پورت TCP `27890` اجرا می‌کند.
2. سرور دسکتاپ درخواست‌های کشف UDP را روی پورت `27891` دریافت می‌کند.
3. برنامه اندروید پیام `NOTIFY_DISCOVER` را در شبکه Wi-Fi ارسال می‌کند.
4. دسکتاپ آدرس‌های LAN قابل استفاده و پورت WebSocket را برمی‌گرداند.
5. برنامه اندروید دستگاه‌های پیدا‌شده را به شکل کارت در بخش **Available devices** نشان می‌دهد.
6. کاربر روی یک دستگاه می‌زند و برنامه همان آدرس را برای اتصال آزمایش می‌کند.
7. اعلان‌های جدید، حذف اعلان، اطلاعات پاسخ سریع، کلیپ‌بورد و telemetry از طریق WebSocket منتقل می‌شوند.
8. دسکتاپ اطلاعات را پردازش، ذخیره و نمایش می‌دهد.

برای همگام‌سازی عادی اعلان‌ها نیازی به ADB نیست. ADB ممکن است برای بعضی قابلیت‌های اختیاری کنترل دستگاه در پروژه وجود داشته باشد، اما مسیر اصلی اعلان‌ها شبکه‌ای است.

---

## پیش‌نیازها

### پیش‌نیازهای دسکتاپ

- Node.js نسخه 18 یا جدیدتر.
- npm؛ فایل `package-lock.json` در پروژه وجود دارد.
- Rust و Cargo نسخه stable.
- پیش‌نیازهای سیستم‌عامل برای Tauri 2.
- سیستم‌عامل پشتیبانی‌شده توسط Tauri.

راهنمای رسمی پیش‌نیازهای Tauri:
<https://v2.tauri.app/start/prerequisites/>

### پیش‌نیازهای اندروید

- Android Studio یا ابزارهای خط فرمان Android SDK.
- Android SDK Platform 36.
- Android Build Tools نسخه `36.0.0`.
- JDK 17.
- Gradle 8.13 یا محیط Gradle سازگار.
- گوشی اندرویدی با Android 7/API 24 یا بالاتر.

تنظیمات فعلی ماژول اندروید:

- حداقل SDK: `24`
- target SDK: `36`
- compile SDK: `36`

گوشی و کامپیوتر معمولاً باید به یک شبکه Wi-Fi یا LAN یکسان متصل باشند.

---

## نصب و اجرای سریع

### ۱. دریافت پروژه

```bash
git clone <repository-url>
cd notify
```

### ۲. نصب وابستگی‌های فرانت‌اند

```bash
npm install
```

### ۳. اجرای برنامه دسکتاپ در حالت توسعه

```bash
npm run tauri dev
```

برای اجرای فقط رابط وب:

```bash
npm run dev
```

### ۴. بررسی build فرانت‌اند

```bash
npm run build
```

### ۵. ساخت و نصب برنامه اندروید

پوشه `android-companion` را در Android Studio باز کنید، SDK را تنظیم کنید و برنامه `app` را روی گوشی اجرا کنید؛ یا از دستور Gradle بخش بعد استفاده کنید.

### ۶. جفت‌سازی

1. Notify را روی کامپیوتر اجرا کنید.
2. برنامه Notify Companion را روی گوشی باز کنید.
3. مطمئن شوید هر دو دستگاه روی یک شبکه هستند.
4. روی **Auto-Discover PC on Wi-Fi** بزنید.
5. از قسمت **Available devices** یک دستگاه را انتخاب کنید.
6. یا از گزینه **Scan PC QR Code** استفاده کنید.
7. مجوز **Notification Access** را فعال کنید.
8. برای اتصال پایدار، سرویس پس‌زمینه و تنظیمات باتری را بررسی کنید.

---

## تنظیمات برنامه اندروید

### مجوزها

برنامه مجوزهای زیر را اعلام می‌کند:

- اینترنت و وضعیت شبکه.
- foreground service و data synchronization.
- اعلان‌های اندروید در نسخه‌های جدید.
- دوربین برای اسکن QR.
- دریافت رویداد روشن‌شدن گوشی.
- درخواست استثنا از بهینه‌سازی باتری.

مهم‌ترین مجوز، **Notification Access** است. بدون این مجوز اندروید اجازه خواندن اعلان‌ها را نمی‌دهد.

برای دسترسی به تنظیمات، روی آیکون چرخ‌دنده در بالای سمت راست برنامه بزنید. مجوز اعلان و تنظیمات باتری در یک پنجره جداگانه نمایش داده می‌شوند.

### اجرای پس‌زمینه

برنامه برای ادامه اتصال از foreground service استفاده می‌کند. بعضی برندهای گوشی برنامه‌های پس‌زمینه را به صورت تهاجمی متوقف می‌کنند. اگر هنگام خاموش شدن صفحه اتصال قطع شد:

- پنجره Settings برنامه را باز کنید.
- بهینه‌سازی باتری Notify Companion را غیرفعال کنید.
- در صورت وجود، اجازه اجرای خودکار و فعالیت پس‌زمینه را فعال کنید.
- برنامه را Force Stop نکنید.

---

## ساخت APK

از پوشه `android-companion` اجرا کنید:

```bash
gradle :app:assembleDebug
```

اگر در ویندوز دستور `gradle` پیدا نشد، از Gradle نصب‌شده در سیستم یا Android Studio استفاده کنید.

مسیر معمول APK:

```text
android-companion/app/build/outputs/apk/debug/app-debug.apk
```

نصب با ADB در صورت اتصال و مجوز داشتن گوشی:

```bash
adb install -r android-companion/app/build/outputs/apk/debug/app-debug.apk
```

برای نسخه release:

```bash
gradle :app:assembleRelease
```

قبل از انتشار عمومی باید کلید امضای release، version code، version name، تنظیمات R8/ProGuard و امنیت ارتباط بررسی شوند. APK debug را برای انتشار عمومی استفاده نکنید.

> ADB در این بخش فقط برای نصب و اشکال‌زدایی استفاده شده است؛ همگام‌سازی عادی اعلان‌ها به ADB نیاز ندارد.

---

## ساخت برنامه دسکتاپ

```bash
npm install
npm run tauri build
```

فایل‌های خروجی بسته به سیستم‌عامل در مسیرهای build تولید می‌شوند. قبل از انتشار بررسی کنید:

- دسکتاپ و گوشی همدیگر را پیدا می‌کنند.
- فایروال TCP پورت `27890` و UDP پورت `27891` را مسدود نکرده باشد.
- محل ذخیره SQLite پایدار باشد.
- tray و autostart مطابق انتظار کار کنند.
- امضای release و مشخصات installer تنظیم شده باشند.

---

## پورت‌ها و تنظیمات مهم

| مورد | مقدار | کاربرد |
| --- | ---: | --- |
| WebSocket دسکتاپ | `27890/TCP` | ارتباط زنده گوشی و دسکتاپ |
| کشف دستگاه | `27891/UDP` | پیدا کردن دسکتاپ در شبکه محلی |
| سرور توسعه Vite | `1420/TCP` | رابط فرانت‌اند در حالت توسعه |
| حداقل Android SDK | `24` | Android 7 و بالاتر |
| شناسه دسکتاپ | `com.notify.desktop` | شناسه برنامه Tauri |
| شناسه اندروید | `com.notify.companion` | application ID اندروید |

سرور دسکتاپ روی همه interfaceها گوش می‌دهد، اما فایروال سیستم‌عامل هنوز ممکن است دسترسی گوشی را مسدود کند.

---

## حریم خصوصی و امنیت

Notify برای نگه‌داشتن اطلاعات اعلان روی شبکه محلی و دستگاه‌های جفت‌شده طراحی شده است؛ با این حال اعلان‌ها ممکن است شامل پیام خصوصی، رمز یک‌بارمصرف، ایمیل و اطلاعات حساس باشند.

نکات مهم:

- سرور WebSocket را مستقیماً روی اینترنت منتشر نکنید.
- پورت‌های `27890` و `27891` را روی روتر port-forward نکنید.
- QR و pairing string را عمومی نکنید.
- فقط از شبکه Wi-Fi مورد اعتماد استفاده کنید.
- مجوز Notification Access اندروید و اعلان‌های دسکتاپ را آگاهانه فعال کنید.
- ارتباط فعلی از `ws://` استفاده می‌کند و برای شبکه محلی مورد اعتماد در نظر گرفته شده است، نه اینترنت عمومی.
- برای انتشار production باید اعتبارسنجی pairing token و ارتباط رمزنگاری‌شده یا احراز هویت‌شده اضافه شود.

---

## رفع اشکال

### دسکتاپ در گوشی پیدا نمی‌شود

1. اتصال هر دو دستگاه به یک شبکه را بررسی کنید.
2. مطمئن شوید Notify در دسکتاپ در حال اجراست.
3. اجازه UDP پورت `27891` را در فایروال بدهید.
4. اجازه TCP پورت `27890` را در فایروال بدهید.
5. AP/client isolation روتر را غیرفعال کنید.
6. VPN را موقتاً خاموش و دوباره تست کنید.
7. اتصال دستی با IP را امتحان کنید.
8. مطمئن شوید گوشی روی Guest Network نیست.

### چند IP عجیب در لیست دیده می‌شود

کامپیوتر ممکن است Wi-Fi، Ethernet، VPN، Docker، VirtualBox، WSL یا Hyper-V داشته باشد. برنامه چند آدرس احتمالی را اعلام می‌کند و گوشی آن‌ها را آزمایش می‌کند. آدرسی را انتخاب کنید که در همان شبکه قابل دسترس گوشی باشد.

### اتصال برقرار است اما اعلان نمی‌آید

1. تنظیمات اندروید را باز کنید.
2. عبارت **Notification access** را جست‌وجو کنید.
3. دسترسی Notify Companion را فعال کنید.
4. برنامه را دوباره باز کنید.
5. وضعیت اتصال دسکتاپ را بررسی کنید.
6. یک اعلان جدید آزمایشی ارسال کنید.

### اتصال در پس‌زمینه قطع می‌شود

در پنجره Settings برنامه، بهینه‌سازی باتری را غیرفعال کنید. همچنین تنظیمات مخصوص برند گوشی برای autostart، background activity و protected apps را بررسی کنید.

### QR اسکن نمی‌شود

- مجوز دوربین را فعال کنید.
- QR را کامل و بدون تاری در تصویر قرار دهید.
- روشنایی دسکتاپ را افزایش دهید.
- از کشف Wi-Fi یا اتصال دستی استفاده کنید.

### دستور Gradle پیدا نمی‌شود

Gradle یا Android Studio را نصب و محیط آن را تنظیم کنید. پروژه فعلاً اسکریپت `gradlew` کامیت‌شده ندارد، بنابراین نام دستور Gradle ممکن است در سیستم‌های مختلف متفاوت باشد.

### Android SDK پیدا نمی‌شود

فایل `android-companion/local.properties` را تنظیم کنید؛ برای نمونه در ویندوز:

```properties
sdk.dir=C\\:\\Users\\<username>\\AppData\\Local\\Android\\Sdk
```

فایل `local.properties` را commit نکنید، چون مسیر SDK مخصوص همان کامپیوتر است.

---

## ساختار پروژه

```text
notify/
├── android-companion/          # برنامه همراه اندروید
├── public/                     # فایل‌های ثابت فرانت‌اند
├── src/                        # رابط React/TypeScript دسکتاپ
├── src-tauri/                  # backend Rust و Tauri
│   ├── src/companion/          # کشف UDP و سرور WebSocket
│   ├── src/notifications/      # پردازش، ذخیره و تشخیص OTP
│   ├── src/adb/                # قابلیت‌های اختیاری ADB
│   ├── src/storage/            # لایه SQLite
│   ├── src/controls/           # کنترل‌های دستگاه
│   └── tauri.conf.json         # تنظیمات بسته‌بندی دسکتاپ
├── package.json
└── README.md
```

---

## نکات توسعه

- تغییرات موبایل و دسکتاپ را تا حد امکان جدا نگه دارید.
- تغییرات رابط اندروید در حال حاضر عمدتاً در `MainActivity.kt` به صورت programmatic انجام می‌شود.
- هنگام تغییر شبکه، حالت VPN، چند کارت شبکه، فایروال و قطع و وصل Wi-Fi را تست کنید.
- هنگام تغییر اعلان‌ها، اعلان جدید، تغییر اعلان، حذف اعلان، OTP، work profile، پاسخ سریع و اعلان‌های تکراری را تست کنید.
- فایل‌های build، کش Gradle و `local.properties` فایل منبع محسوب نمی‌شوند و معمولاً نباید در تغییرات کد بررسی شوند.

---

## مسیر توسعه پیشنهادی

- افزودن Gradle wrapper برای build قابل تکرار.
- تست خودکار اتصال و UI اندروید.
- اعتبارسنجی واقعی pairing token در سرور.
- افزودن TLS یا transport رمزنگاری‌شده.
- پشتیبانی از چند گوشی هم‌زمان.
- صفحه مدیریت دستگاه‌های ذخیره‌شده.
- فیلتر اعلان بر اساس برنامه.
- پاسخ سریع و actionهای کامل‌تر.
- APK release و installer دسکتاپ امضاشده از طریق CI.
- بررسی دسترس‌پذیری رابط‌ها.

---

## مشارکت در پروژه

1. یک branch اختصاصی بسازید.
2. تغییرات موبایل و دسکتاپ را در صورت امکان جدا نگه دارید.
3. conventions موجود TypeScript، Rust و Kotlin را رعایت کنید.
4. قبل از Pull Request بررسی‌های مرتبط را اجرا کنید:

```bash
npm run build
cd src-tauri && cargo check
cd ../android-companion && gradle :app:assembleDebug
```

5. هنگام گزارش مشکل اتصال، نسخه اندروید و شرایط شبکه را ذکر کنید.
6. اعلان‌های واقعی، pairing secret، کلید امضای APK و مسیر SDK را در issue یا commit قرار ندهید.

---

## مجوز

در حال حاضر فایل license در repository وجود ندارد. تا زمان اضافه‌شدن license، تمام حقوق برای مالک اثر محفوظ است. پیش از بازتوزیع یا دریافت مشارکت عمومی، یک license صریح به پروژه اضافه کنید.
