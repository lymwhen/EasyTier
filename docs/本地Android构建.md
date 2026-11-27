# 本地 Android 构建

EasyTier Tauri v2 Android App 本地构建指南，覆盖 Ubuntu/WSL 与 Windows。

> CI 全平台构建请参阅 [docs/CI全平台构建.md](CI全平台构建.md)。

---

## Windows 快速构建（当前机器推荐）

Windows 构建不要复用 WSL 生成的 `tauri.settings.gradle`。该文件包含 Linux 绝对路径，会让 Gradle 报“Tauri 子工程没有 variant”。仓库脚本会从 `Cargo.lock` 自动解析版本并重新生成 Windows 路径。

```powershell
cd D:\projects\projectsAlpha\EasyTier
.\script\build-android-windows.ps1
```

常用参数：

| 参数 | 用途 |
|------|------|
| `-ValidateOnly` | 只检查 JDK/SDK/NDK/Rust/MinGW、生成 Windows Gradle 设置，约 1 秒 |
| `-InstallDeps` | 重新执行 Windows `pnpm install --frozen-lockfile` |
| `-BuildFrontendDeps` | `tauri-plugin-vpnservice` 或 `frontend-lib` 改动时额外构建两个前置包 |
| `-CleanAndroid` | Gradle 打包前执行 `:app:clean` |
| `-OutputDir D:\build\apk` | 自定义 APK 输出目录，默认 `F:\tmp\apk` |

脚本固定使用：

- JDK 17：`D:\tools\Java\jdk-17.0.12`
- Android SDK：`D:\tools\Android\Sdk`
- NDK：`27.0.12077973`
- Rust：`stable-x86_64-pc-windows-gnu`
- MinGW：`C:\Users\lymly\mingw64\mingw64`
- Cargo registry：`C:\Users\lymly\.cargo`

如果 GNU Android target 缺失，脚本会通过 `rsproxy.cn` 自动安装。脚本最后会完成 APK 签名验证、前端资源哈希校验、Native 库哈希校验并输出 SHA-256。

本次 2026-11-26 冷构建实测：

| 阶段 | 耗时 |
|------|------|
| Windows Vite | 1 分 50 秒 |
| Windows GNU → Android arm64 Cargo（含首次依赖编译/链接） | 10 分 09 秒 |
| Gradle `assembleArm64Release` | 2 分 19 秒 |
| 合计 | 约 15 分钟 |

第二次构建通常只需 2–5 分钟。若耗时再次接近 15 分钟，优先检查是否切换了 Rust toolchain、node_modules 是否被另一种操作系统覆盖，或 Rustup 是否在重新下载组件。

---

## Ubuntu / WSL 快速构建（工具缓存已配置）

当前 Ubuntu/WSL 环境已经安装并缓存 Rust、JDK、Android SDK/NDK、protoc、pnpm 和 Gradle 依赖。重新开始会话时，先加载环境：

```bash
source /mnt/d/tools/EasyTierLinux/env.sh
cd /mnt/d/projects/projectsalpha/EasyTier
```

### 工具和缓存位置

| 项目 | 路径 |
|------|------|
| Ubuntu 构建环境 | `/mnt/d/tools/EasyTierLinux/env.sh` |
| 一键构建脚本 | `/mnt/d/tools/EasyTierLinux/build-android-arm64.sh` |
| JNI 桥生成脚本 | `/mnt/d/tools/EasyTierLinux/generate-wry-bridge.sh` |
| JNI 桥校验脚本 | `/mnt/d/tools/EasyTierLinux/verify-android-jni.sh` |
| Rust / Cargo | `/mnt/d/tools/rust-linux` |
| JDK 17 | `/mnt/d/tools/Java/jdk-17.0.21-linux` |
| Linux Android SDK | `/mnt/d/tools/Android/SdkLinux` |
| Linux NDK 原始包 | `/mnt/d/tools/Android/ndk-archives/android-ndk-r27-linux.zip` |
| protoc 35.1 | `/mnt/d/tools/EasyTierLinux/protoc` |
| Ubuntu 主机 C 编译器 | `/mnt/d/tools/EasyTierLinux/bin/cc` |
| pnpm 9.12.1 | `/mnt/d/tools/EasyTierLinux/bin/pnpm` |
| Gradle 用户缓存 | `/mnt/f/tmp/gradle-tmp` |
| APK 输出目录 | `/mnt/f/tmp/apk` |

Linux NDK 不要直接解压到 NTFS 目录。NDK 中存在 8 组仅大小写不同的头文件，NTFS 默认无法同时保存；因此 D 目录保留原始 ZIP，`SdkLinux/ndk/.../linux-x86_64` 指向 ext4/home 下的解压缓存。

普通 Ubuntu shell 通常找不到 `java` 和 `pnpm`；执行手动步骤前先加载环境并做快速检查：

```bash
source /mnt/d/tools/EasyTierLinux/env.sh
command -v pnpm cargo java
test -x /mnt/d/tools/EasyTierLinux/build-android-arm64.sh
```

`build-android-arm64.sh` 内部也会加载同一个 `env.sh`，因此一键脚本不需要依赖当前 shell 是否预先执行过 `source`；但 Vite、Cargo 和 Gradle 必须保持同一环境变量快照。

### 一键构建

代码没有依赖清单变化时，直接执行：

```bash
source /mnt/d/tools/EasyTierLinux/env.sh
/mnt/d/tools/EasyTierLinux/build-android-arm64.sh
```

脚本会依次完成：

1. 构建 `tauri-plugin-vpnservice` 和 `easytier-web/frontend-lib`
2. 构建 `easytier-gui` 前端
3. 将 `easytier-gui/dist/` 同步到 Android `assets/frontend/`
4. 交叉编译 `libapp_lib.so`
5. 校验 Android Kotlin JNI 桥和 Native 导出
6. 执行 Gradle `assembleArm64Release`
7. 将带时间戳的 APK 复制到 `/mnt/f/tmp/apk`

可选环境变量：

```bash
# 强制重新安装 pnpm 依赖
INSTALL_DEPS=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh

# 前端依赖没有变化，跳过 plugin/frontend-lib 构建
SKIP_FRONTEND_DEPS=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh

# 跳过 Cargo；仅在 Rust 与前端产物均未变化时使用
SKIP_RUST=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh

# 清理 Android 构建缓存后再构建
CLEAN_ANDROID=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh
```

### 如何选择快速构建

| 变更范围 | 推荐命令 | 构建行为 |
|------|------|------|
| 仅 `easytier-gui` 的 Vue/TS/CSS | `SKIP_FRONTEND_DEPS=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh` | 仍执行 Vite、Cargo 和 Gradle |
| `tauri-plugin-vpnservice`、`frontend-lib` 或 lockfile 变化 | 不设置 `SKIP_FRONTEND_DEPS` | 构建前端依赖，再执行 Vite、Cargo 和 Gradle |
| 仅 Rust Native 变化 | `SKIP_FRONTEND_DEPS=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh` | 跳过前端依赖，重新编译 Rust 并打包 |
| 仅 Android Kotlin/Manifest 变化，且前端与 Rust 产物不变 | `SKIP_FRONTEND_DEPS=1 SKIP_RUST=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh` | 不编译前端依赖和 Rust；仍执行 Vite 资源同步、JNI 校验与 Gradle 打包 |

> ⚠️ **前端修改不能设置 `SKIP_RUST=1`。** Tauri 会把 `easytier-gui/dist/` 嵌入 `libapp_lib.so`；只运行 Vite 和 Gradle 会导致 APK 仍携带旧的 Native 内嵌资源。`SKIP_FRONTEND_DEPS=1` 只跳过 plugin/frontend-lib，不会跳过 `easytier-gui` 的 Vite 构建。

### 构建结果与修改验收

一键脚本成功时应同时出现以下关键输出：

```text
bridge=22 native=22 missing=[] extra=[]
Android JNI bridge matches libapp_lib.so
BUILD SUCCESSFUL
APK: /mnt/f/tmp/apk/app-arm64-release-YYYYMMDDHHMM.apk
<sha256>  /mnt/f/tmp/apk/app-arm64-release-YYYYMMDDHHMM.apk
```

脚本已经自动执行 `apksigner verify`；出现 `APK:` 和 SHA-256 表示签名校验通过。仅看到 Vite 构建成功不代表最终 APK 已更新，还必须看到 Cargo、JNI 校验和 Gradle 成功。

如果需要确认某项前端修改确实进入 APK，应使用修改产生的唯一字符串进行检查。Ubuntu 环境可能没有 `unzip`，使用 Python 标准库即可：

```bash
APK=/mnt/f/tmp/apk/app-arm64-release-YYYYMMDDHHMM.apk
MARKER=本次修改引入的唯一字符串
python3 - "$APK" "$MARKER" <<'PY'
import sys
import zipfile

apk, marker = sys.argv[1], sys.argv[2].encode()
with zipfile.ZipFile(apk) as zf:
    hits = [
        name for name in zf.namelist()
        if name.startswith("assets/frontend/") and marker in zf.read(name)
    ]

print(f"marker={marker.decode()}")
print(f"hits={hits}")
assert hits, "修改未出现在 APK 的 assets/frontend 中"
PY
```

验证后恢复临时测试内容，并重新执行快速构建，确认最终 APK 不再包含测试标记。常规修改也可以比较修改前后的 APK SHA-256，并检查 APK 内 `assets/frontend/` 与 `app/src/main/assets/frontend/`、`libapp_lib.so` 与 `jniLibs/arm64-v8a/libapp_lib.so` 是否逐文件一致。

### 手动构建步骤

**1. 安装依赖（仅首次、依赖变化或 `node_modules` 缺失时）**

```bash
source /mnt/d/tools/EasyTierLinux/env.sh
cd /mnt/d/projects/projectsalpha/EasyTier/easytier-gui
pnpm install --frozen-lockfile --ignore-scripts
```

**2. 构建前端依赖和 GUI 前端**

当前定制的 `easytier-gui` 构建命令是 `vite build`，不会自动构建 `frontend-lib`，因此必须显式构建：

```bash
cd /mnt/d/projects/projectsalpha/EasyTier
pnpm --dir tauri-plugin-vpnservice build
pnpm --dir easytier-web/frontend-lib build
pnpm --dir easytier-gui build
```

**3. 交叉编译 Rust Native Library**

```bash
source /mnt/d/tools/EasyTierLinux/env.sh
cd /mnt/d/projects/projectsalpha/EasyTier

cargo build \
  --manifest-path easytier-gui/src-tauri/Cargo.toml \
  --package easytier-gui \
  --lib \
  --release \
  --target aarch64-linux-android \
  --features 'tauri/custom-protocol' \
  --locked \
  --config 'source.crates-io.replace-with="ustc"' \
  --config 'source.ustc.registry="sparse+https://mirrors.ustc.edu.cn/crates.io-index/"'
```

USTC 镜像用于首次下载 Cargo 依赖；已有缓存时不影响增量构建。

**4. 校验 Android JNI 桥**

```bash
source /mnt/d/tools/EasyTierLinux/env.sh
cd /mnt/d/projects/projectsalpha/EasyTier

/mnt/d/tools/EasyTierLinux/verify-android-jni.sh \
  "$PWD" \
  "$PWD/target/aarch64-linux-android/release/libapp_lib.so"
```

正常输出：

```text
bridge=22 native=22 missing=[] extra=[]
Android JNI bridge matches libapp_lib.so
```

如果 `Rust.kt` 缺失或校验失败，必须先重新生成 JNI 桥：

```bash
/mnt/d/tools/EasyTierLinux/generate-wry-bridge.sh "$PWD"
/mnt/d/tools/EasyTierLinux/verify-android-jni.sh \
  "$PWD" \
  "$PWD/target/aarch64-linux-android/release/libapp_lib.so"
```

> ⚠️ **关键：不要直接复用旧的 `gen/android/.../generated` 文件。** Gradle 使用 `-x rustBuildArm64Release` 时不会执行 Tauri/Wry codegen。旧 Kotlin JNI 类与新 `libapp_lib.so` 不匹配会导致点击图标立即闪退。生成版本必须与 `Cargo.lock` 中的 `wry` 版本一致，当前为 `wry 0.55.1`。

**5. 复制 Native Library 并执行 Gradle**

```bash
ROOT=/mnt/d/projects/projectsalpha/EasyTier
cp "$ROOT/target/aarch64-linux-android/release/libapp_lib.so" \
  "$ROOT/easytier-gui/src-tauri/gen/android/app/src/main/jniLibs/arm64-v8a/libapp_lib.so"

# Gradle 使用 -x rustBuildArm64Release 时不会同步 Tauri 前端资源
ASSETS="$ROOT/easytier-gui/src-tauri/gen/android/app/src/main/assets/frontend"
rm -rf "$ASSETS"
mkdir -p "$ASSETS"
cp -a "$ROOT/easytier-gui/dist/." "$ASSETS/"

cd "$ROOT/easytier-gui/src-tauri/gen/android"
./gradlew assembleArm64Release --no-daemon -x rustBuildArm64Release
```

如果怀疑 APK 使用了旧 Native/JNI 文件，清理后再构建：

```bash
./gradlew :app:clean assembleArm64Release --no-daemon -x rustBuildArm64Release
```

**6. 复制 APK**

```bash
ROOT=/mnt/d/projects/projectsalpha/EasyTier
ts="$(date +%Y%m%d%H%M)"
cp "$ROOT/easytier-gui/src-tauri/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release.apk" \
  "/mnt/f/tmp/apk/app-arm64-release-$ts.apk"
```

### Lockfile 处理原则

- `Cargo.lock`：优先以上游分支的 `Cargo.lock` 为基线，只针对定制依赖做最小解析。不要直接 `cargo generate-lockfile` 全量重建；当前项目曾因此解析出不兼容的 `rust-i18n 4.0.x` / `rust-i18n-support 4.2.x` 组合，导致 Rust 编译失败。
- `pnpm-lock.yaml`：使用 pnpm 9.12.1 更新，不要混用 pnpm 12 生成：

  ```bash
  source /mnt/d/tools/EasyTierLinux/env.sh
  cd /mnt/d/projects/projectsalpha/EasyTier/easytier-gui
  pnpm install --lockfile-only --no-frozen-lockfile
  ```

- `androidx.lifecycle:lifecycle-process` 是 Wry `ProcessLifecycleOwner` 所需依赖，不应删除。

### Ubuntu/WSL 构建耗时参考

以下为 2026-11-23 在当前 WSL、缓存已存在环境中的实测值；NTFS/D 盘上的 Vite 与 Cargo 阶段明显慢于文档后半部分的 Windows 参考值：

| 场景 | 命令 | 实测耗时 |
|------|------|------|
| 完整一键构建，包含 plugin/frontend-lib | 默认命令 | 约 13 分 45 秒 |
| 仅 `easytier-gui` 前端修改 | `SKIP_FRONTEND_DEPS=1` | 约 6 分 53 秒 |
| 其中 Vite | — | 约 2 分 25 秒 |
| 其中 Cargo `easytier-gui` 增量/重链 | — | 约 2 分 29 秒 |
| 其中 Gradle | — | 约 47–67 秒 |
| 全新环境、冷缓存 | 默认命令 | 主要耗时在 Cargo/Gradle 下载和首次编译 |

预期成功但不需要中断构建的常见输出：

- Browserslist `caniuse-lite is 6 months old`
- Android Gradle Plugin 8.3.2 对 `compileSdk = 36` 的兼容性提示
- Rust 的 unused import / dead code 警告
- `lintVital` 阶段的 Kotlin metadata `2.1.0` 与 `1.9.0` 不匹配日志；只要最后为 `BUILD SUCCESSFUL` 即不阻断构建

缓存已经配置在 `/mnt/d/tools`、`/mnt/f/tmp/gradle-tmp` 和 Cargo/Rust 缓存目录。正常增量构建不要再重新下载工具链。

---

## 一、前置：加载 `/mnt/d/tools/EasyTierLinux/env.sh`

不要把本机绝对路径写入仓库中的 `.cargo/config.toml`。这些路径只属于当前 Ubuntu/WSL 构建环境；如果提交到仓库，会污染 GitHub Actions 的 `PROTOC`、`LIBCLANG_PATH` 和 Android 交叉编译工具链，导致所有 Rust、Web、OHOS、Nix 工作流失败。

统一通过环境脚本注入：

```bash
source /mnt/d/tools/EasyTierLinux/env.sh
```

### 环境变量来源

| 变量 | env.sh 中的来源 | 用途 |
|------|-----------------|------|
| `CC_aarch64_linux_android` / `AR_*` / `BINDGEN_EXTRA_CLANG_ARGS_*` | `/mnt/d/tools/Android/SdkLinux/ndk/...` | Android NDK 交叉编译（kcp-sys、ring 等 C 依赖） |
| `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER` | Linux NDK clang | Android Rust 链接 |
| `PROTOC` | `/mnt/d/tools/EasyTierLinux/protoc/bin/protoc` | prost-wkt-types build.rs |
| `LIBCLANG_PATH` | Linux NDK musl libclang | kcp-sys bindgen |
| `GIT_CEILING_DIRECTORIES` | 当前仓库路径 | 阻止 git-version proc-macro 读取 `.git/` 文件，保持 Cargo 指纹稳定 |

### CI 隔离要求

- `.cargo/config.toml` 只能保留上游的跨平台目标配置。
- 本机工具链路径由 `env.sh`、构建脚本或命令行 `--config` 注入。
- 不再使用 `git update-index --skip-worktree .cargo/config.toml` 隐藏宿主机配置。
- 如果需要临时增加 Cargo 配置，优先使用 `cargo --config KEY=VALUE`，避免修改共享配置文件。

---

## 二、构建命令

### Windows 一键流程

正常构建只执行：

```powershell
cd D:\projects\projectsAlpha\EasyTier
.\script\build-android-windows.ps1
```

脚本内部按顺序执行：

1. 校验 JDK 17、SDK/NDK、MinGW、protoc、libclang 和 Rust target。
2. 根据 `Cargo.lock` 定位 Tauri/插件源码，重新生成 Windows `tauri.settings.gradle`。
3. 构建 `easytier-gui/dist`。
4. 使用 `stable-x86_64-pc-windows-gnu` 和 NDK 27 交叉编译 `libapp_lib.so`。
5. 同步 `dist` 到 Android assets，同步 `.so` 到 `jniLibs`，清理三个旧 Native 中间目录。
6. 使用 JDK 17 通过 `cmd.exe /d /c` 执行 `gradlew.bat assembleArm64Release --no-daemon --exclude-task=rustBuildArm64Release`。PowerShell 7.7+ 直接调用 `.bat` 时可能错误截断参数并报 `Task 's' is ambiguous`，一键脚本已固定通过 `cmd.exe` 调用。
7. 复制带时间戳的 APK，并验证签名、前端资源和 Native 库哈希。

仅验证环境：

```powershell
.\script\build-android-windows.ps1 -ValidateOnly
```

### Windows 手动排查

只有脚本失败时才需要手工拆分。关键顺序不可变：Vite → Cargo → 同步资源 → Gradle。

```powershell
$env:JAVA_HOME = 'D:\tools\Java\jdk-17.0.12'
$env:ANDROID_HOME = 'D:\tools\Android\Sdk'
$env:NDK_HOME = 'D:\tools\Android\Sdk\ndk\27.0.12077973'
$env:GRADLE_USER_HOME = 'D:\tmp\gradle-tmp'
$env:Path = "$env:JAVA_HOME\bin;$env:NDK_HOME\toolchains\llvm\prebuilt\windows-x86_64\bin;$env:Path"

corepack pnpm@9.12.1 --dir easytier-gui run build
```

Cargo 必须使用 GNU toolchain，不能使用默认的 MSVC toolchain，因为本机没有 Visual Studio Build Tools：

```powershell
$env:CC_aarch64_linux_android = "$env:NDK_HOME\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android24-clang.cmd"
$env:AR_aarch64_linux_android = "$env:NDK_HOME\toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-ar.exe"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $env:CC_aarch64_linux_android
$env:LIBCLANG_PATH = 'C:\Users\lymly\llvm-dll'
$env:PROTOC = 'C:\Users\lymly\protoc\bin\protoc.exe'
$env:GIT_CEILING_DIRECTORIES = 'D:\projects\projectsAlpha\EasyTier'

rustup run stable-x86_64-pc-windows-gnu cargo build `
  --manifest-path easytier-gui/src-tauri/Cargo.toml `
  --package easytier-gui `
  --lib --release --target aarch64-linux-android `
  --features 'tauri/custom-protocol' --locked
```

Gradle 前必须同步资源和 Native 库；脚本已自动完成。手动执行时：

```powershell
Copy-Item -Force `
  'target\aarch64-linux-android\release\libapp_lib.so' `
  'easytier-gui\src-tauri\gen\android\app\src\main\jniLibs\arm64-v8a\libapp_lib.so'

$assets = 'easytier-gui\src-tauri\gen\android\app\src\main\assets\frontend'
Remove-Item -Recurse -Force $assets -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $assets | Out-Null
Copy-Item -Path 'easytier-gui\dist\*' -Destination $assets -Recurse -Force

Set-Location 'easytier-gui\src-tauri\gen\android'
cmd.exe /d /c ".\gradlew.bat assembleArm64Release --no-daemon --exclude-task=rustBuildArm64Release"
```

### 构建耗时

Windows 与 Ubuntu 的耗时不同。上表位于本文顶部；以 2026-11-26 同机实测为准：

| 场景 | 预期 |
|------|------|
| 环境验证 `-ValidateOnly` | < 1 秒 |
| 第二次无缓存变化完整构建 | 2–5 分钟 |
| 首次 Windows 交叉编译 + 全部 Cargo 依赖 | 约 15 分钟 |
| 仅 Android Kotlin/资源改动，前端和 Rust 未变 | 可跳过 Cargo，但仍需脚本完成资源同步 |

---

## 三、关键原理

### 为什么 Vite、Cargo、Gradle 缺一不可

Tauri 的 `tauri_build::build()` / `generate_context!` 会嵌入 `dist/` 前端产物；手动 Gradle 使用 `-x rustBuildArm64Release` 时还会跳过 Android `assets/frontend/` 同步。Windows 与 Ubuntu 脚本都会显式同步 assets。前端变更必须重新执行 Cargo 并按完整流程构建，再检查 APK 中的资源哈希，避免 `.so`、生成资源和 Gradle 缓存之间版本不一致。

### 为什么 Vite + Cargo 必须使用同一套 Windows 环境

Cargo build.rs 会通过 `rerun-if-env-changed` 跟踪 `PROTOC`、`LIBCLANG_PATH` 和交叉编译变量。不同终端或不同路径格式会导致依赖链全量重编。

Windows 脚本在单个 PowerShell 进程中设置 JDK、NDK、GNU toolchain、MinGW、protoc 和 libclang，再依次执行 Vite 与 Cargo；这样 Cargo 指纹稳定，也避免混用 WSL 的 Linux Cargo 缓存。

### 为什么 Gradle 必须用 PowerShell

bash (MSYS2) 文件操作与 Windows NTFS 文件锁冲突，Gradle transforms 阶段报 `Could not move temporary workspace`。PowerShell 使用 Windows 原生 API 无此问题。

### 为什么跳过 Gradle 的 rustBuildArm64Release

`rustBuildArm64Release` 内部调用 `pnpm tauri android android-studio-script`，依赖 `%TEMP%\com.kkrainbow.easytier-server-addr` 临时文件连接 Tauri CLI TCP 服务端。该服务端仅 Android Studio 启动时存在，命令行构建中不存在 → Tauri CLI panic。脚本已提前通过 `cargo build --lib` 生成 `.so`，再手动复制并用 `-x` 跳过该任务。

清理 `merged_jni_libs` / `merged_native_libs` / `stripped_native_libs` 确保 Gradle 不使用缓存的旧 `.so`。

### 指纹稳定性

Cargo 增量编译依赖"指纹"(fingerprint) 稳定。以下措施保证指纹不因无关因素失效：

1. **所有交叉编译 env var 由 Windows 脚本一次性注入** — Vite、Cargo、Gradle 使用同一路径格式和 toolchain
2. **Vite + Cargo 在同一 PowerShell 脚本中执行** — 单次环境快照
3. **`GIT_CEILING_DIRECTORIES`** — easytier-core 的 `constants.rs` 使用 `git_version::git_version!()` proc-macro，编译时运行 `git describe` 并生成 `include_bytes!(".git/index")`。Cargo 基于 mtime 追踪该文件 → 任何 git 操作（含 IDE 后台刷新）都会更新 `.git/index` mtime → easytier-core 指纹失效。设置此变量后 proc-macro 找不到 git 仓库 → 走 fallback（`CARGO_PKG_VERSION`）→ 不生成 git 文件追踪 → 指纹稳定

开发构建的版本号因此显示纯 `CARGO_PKG_VERSION`（如 `2.7.0`），无 commit hash。CI 不受影响。

---

## 四、工具链

| 工具 | 路径 | 用途 |
|------|------|------|
| Rust 宿主 toolchain | `stable-x86_64-pc-windows-gnu` | 替代 MSVC，提供宿主构建脚本链接器 |
| Android target | `aarch64-linux-android` | GNU toolchain 的 Android 标准库 |
| MinGW-w64 | `C:\Users\lymly\mingw64\mingw64` | GCC/ld 宿主链接器 |
| LLVM/libclang | `C:\Users\lymly\llvm-dll`（`libclang.dll`） | kcp-sys bindgen |
| protoc | `C:\Users\lymly\protoc\bin\protoc.exe` | prost-wkt-types |
| pnpm | `corepack pnpm@9.12.1` | 前端包管理，避免 pnpm 版本漂移 |
| JDK 17 | `D:\tools\Java\jdk-17.0.12` | Android Gradle |
| Android SDK | `D:\tools\Android\Sdk` | Android 编译 |
| Android NDK | `27.0.12077973` | arm64 Clang、sysroot、llvm-ar |

---

## 五、踩坑速查

| 问题 | 原因 | 解决 |
|------|------|------|
| `link.exe` 找不到 | 默认 Rust 是 MSVC toolchain，但本机没有 VS Build Tools | 使用 `stable-x86_64-pc-windows-gnu`；脚本自动切换 |
| Rustup 下载 Android target 卡住且 `.partial` 为 0 字节 | 官方源连接异常 | 设置 `RUSTUP_DIST_SERVER=https://rsproxy.cn`；脚本自动使用 |
| Gradle 报 Tauri 子工程 “No matching variant / No variants exist” | `tauri.settings.gradle` 残留 `/home/lymly`、`/mnt/d` WSL 路径 | 运行 Windows 脚本，自动从 `Cargo.lock` 重建为 `C:/...`、`D:/...` |
| Windows 下 Rollup 缺 `@rollup/rollup-win32-x64-msvc` | `node_modules` 被 WSL/Linux 安装覆盖 | 不要跨系统共用 `node_modules`，执行 `-InstallDeps` 重新安装 |
| WSL 下 Rollup 缺 `@rollup/rollup-linux-x64-gnu` | `node_modules` 被 Windows 安装覆盖 | 在对应平台重新安装依赖，或优先使用目标平台的一键脚本 |
| 仅改前端却全量重编 | git-version proc-macro 追踪 `.git/index` mtime | 保持 `GIT_CEILING_DIRECTORIES=仓库根目录`；Windows 脚本已设置 |
| 跨会话 Cargo 全量重编 | toolchain、NDK 路径或 `node_modules` 所属平台变化 | 固定使用 Windows 脚本和 GNU toolchain |
| Gradle `Could not move temporary workspace` | bash + NTFS 文件锁冲突 | 用 PowerShell，设 `$env:GRADLE_USER_HOME` |
| `libclang.dll` / `protoc` 找不到 | kcp-sys / prost-wkt-types 构建需要 | 安装 LLVM / protoc 到 `$HOME` |
| APK 不更新前端 | 前端嵌入 `.so`，且 Gradle 跳过 `rustBuild` 时不会同步 `assets/frontend/` | 前端变更必须跑 Cargo + Vite，同步资源并检查 APK 资源哈希 |
| `usesCleartextTraffic` 不生效 | Release 默认 false | Manifest 硬编码 + manifestPlaceholders |
| Windows 符号链接失败 | 非管理员无权限 | BuildTask.kt 改用文件复制 |
| Gradle 缓存损坏 | JDK 切换或磁盘问题 | 删 `~/.gradle/caches` |
| Kotlin incremental cache “different roots” 并 fallback 编译 | Tauri 插件源码在 C 盘、Android 工程在 D 盘，Kotlin 增量缓存无法跨盘相对化 | 只要最终 `BUILD SUCCESSFUL` 可忽略；必要时执行 `-CleanAndroid` |
