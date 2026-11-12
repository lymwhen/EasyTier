# 本地 Android 构建

EasyTier Tauri v2 Android App 本地构建指南，覆盖 Ubuntu/WSL 与 Windows。

> CI 全平台构建请参阅 [docs/CI全平台构建.md](CI全平台构建.md)。

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
| APK 输出目录 | `/mnt/f/tmp` |

Linux NDK 不要直接解压到 NTFS 目录。NDK 中存在 8 组仅大小写不同的头文件，NTFS 默认无法同时保存；因此 D 目录保留原始 ZIP，`SdkLinux/ndk/.../linux-x86_64` 指向 ext4/home 下的解压缓存。

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
7. 将带时间戳的 APK 复制到 `/mnt/f/tmp`

可选环境变量：

```bash
# 强制重新安装 pnpm 依赖
INSTALL_DEPS=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh

# 前端依赖没有变化，跳过 plugin/frontend-lib 构建
SKIP_FRONTEND_DEPS=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh

# 仅重新打包，且 Rust 与前端产物均未变化
SKIP_RUST=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh

# 清理 Android 构建缓存后再构建
CLEAN_ANDROID=1 /mnt/d/tools/EasyTierLinux/build-android-arm64.sh
```

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
  "/mnt/f/tmp/app-arm64-release-$ts.apk"
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

| 场景 | 预计耗时 |
|------|----------|
| 无变更，只重新打包 | 数分钟 |
| 仅 Android Kotlin/Manifest 变更，`SKIP_RUST=1` | 约 2–4 分钟 |
| 仅 Rust Native 变更 | Rust 增量编译 + Gradle 打包 |
| 前端变更 | Vite 构建 + Gradle 打包 |
| 全新环境、冷缓存 | 主要耗时在 Cargo/Gradle 下载和首次编译 |

缓存已经配置在 `/mnt/d/tools`、`/mnt/f/tmp/gradle-tmp` 和 Cargo/Rust 缓存目录。正常增量构建不要再重新下载工具链。

---

## 一、前置：配置文件 `.cargo/config.toml`

构建依赖以下配置。该文件含本机绝对路径，通过 `skip-worktree` 隔离，不影响远端仓库和 CI：

```toml
[env]
CC_aarch64_linux_android = "D:/tools/Android/Sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android21-clang.cmd"
AR_aarch64_linux_android = "D:/tools/Android/Sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/windows-x86_64/bin/llvm-ar.exe"
BINDGEN_EXTRA_CLANG_ARGS_aarch64_linux_android = "-resource-dir=D:/tools/Android/Sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/windows-x86_64/lib/clang/18 --sysroot=D:/tools/Android/Sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/windows-x86_64/sysroot -ID:/tools/Android/Sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/windows-x86_64/sysroot/usr/include"
PROTOC = "C:/Users/lymly/protoc/bin/protoc.exe"
LIBCLANG_PATH = "C:/Users/lymly/llvm-dll"
GIT_CEILING_DIRECTORIES = "D:/projects/projectsAlpha/EasyTier"

[target.aarch64-linux-android]
linker = "D:/tools/Android/Sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android21-clang.cmd"
ar = "D:/tools/Android/Sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/windows-x86_64/bin/llvm-ar.exe"
```

### 各变量作用

| 变量 | 用途 |
|------|------|
| `CC_aarch64_linux_android` / `AR_*` / `BINDGEN_EXTRA_CLANG_ARGS_*` | Android NDK 交叉编译（kcp-sys, ring 等 C 依赖） |
| `PROTOC` | prost-wkt-types build.rs |
| `LIBCLANG_PATH` | kcp-sys bindgen |
| `GIT_CEILING_DIRECTORIES` | 阻止 git-version proc-macro 读取 `.git/` 文件（见下文"指纹稳定性"） |
| `[target.aarch64-linux-android]` | linker、ar 路径 |

### skip-worktree 隔离

```bash
git update-index --skip-worktree .cargo/config.toml
```

效果：`git status`/`git diff` 不显示该文件变动，`git pull`/`git merge` 不覆盖本地版本。

上游更新此文件时（低频，几个月一次）：
```bash
git update-index --no-skip-worktree .cargo/config.toml
git checkout .cargo/config.toml               # 获取上游新版
# 手动将上述 [env] + [target] 块合并进去
git update-index --skip-worktree .cargo/config.toml
```

---

## 二、构建命令

### 三步流程

**Step 1 — Vite + Cargo（bash，必须同一调用）**

```bash
export PATH="$HOME/.cargo/bin:$HOME/.npm-global:$PATH"
cd easytier-gui && pnpm vite build && cd src-tauri && cargo build --lib --release --target aarch64-linux-android --features "tauri/custom-protocol"
```

**Step 2 — Gradle（必须 PowerShell，不可 bash）**

```powershell
$env:ANDROID_HOME = "D:/tools/Android/Sdk"
$env:JAVA_HOME = "D:/tools/Java/jdk-17.0.12"
$env:GRADLE_USER_HOME = "D:/tmp/gradle-tmp"
Copy-Item -Force "D:\projects\projectsAlpha\EasyTier\target\aarch64-linux-android\release\libapp_lib.so" "D:\projects\projectsAlpha\EasyTier\easytier-gui\src-tauri\gen\android\app\src\main\jniLibs\arm64-v8a\libapp_lib.so"
Remove-Item -Recurse -Force "D:\projects\projectsAlpha\EasyTier\easytier-gui\src-tauri\gen\android\app\build\intermediates\merged_jni_libs" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "D:\projects\projectsAlpha\EasyTier\easytier-gui\src-tauri\gen\android\app\build\intermediates\merged_native_libs" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "D:\projects\projectsAlpha\EasyTier\easytier-gui\src-tauri\gen\android\app\build\intermediates\stripped_native_libs" -ErrorAction SilentlyContinue
Set-Location "D:\projects\projectsAlpha\EasyTier\easytier-gui\src-tauri\gen\android"
.\gradlew.bat assembleArm64Release --no-daemon -x rustBuildArm64Release
```

**Step 3 — 复制 APK**

APK 文件名添加年月日时分后缀（`yyyyMMddHHmm`），便于区分多次构建产物：

```powershell
$ts = Get-Date -Format 'yyyyMMddHHmm'
Copy-Item -Force "D:\projects\projectsAlpha\EasyTier\easytier-gui\src-tauri\gen\android\app\build\outputs\apk\arm64\release\app-arm64-release.apk" "F:\tmp\app-arm64-release-$ts.apk"
```

### 构建耗时

| 变更范围 | Vite | Cargo easytier | Cargo easytier-gui | Gradle | 总耗时 |
|----------|:---:|:---:|:---:|:---:|------|
| 仅前端（Vue/TS/CSS） | ~55s | 缓存命中 | 重链 ~40s | ~12s | **~1m45s** |
| easytier-gui Rust | ~55s | 缓存命中 | 重编 ~1m | ~12s | ~2m10s |
| easytier-core 或其依赖 | ~55s | 重编 ~7m | 重编 ~1m | ~12s | **~10min** |
| 无任何变更 | ~55s | 缓存命中 | 缓存命中 | ~12s | ~1m05s |

---

## 三、关键原理

### 为什么三步缺一不可

Tauri 的 `tauri_build::build()` / `generate_context!` 会嵌入 `dist/` 前端产物；手动 Gradle 使用 `-x rustBuildArm64Release` 时还会跳过 Android `assets/frontend/` 同步。Ubuntu 一键脚本已显式同步 assets。前端变更必须重新执行 Cargo 并按完整流程构建，再检查 APK 中的资源哈希，避免 `.so`、生成资源和 Gradle 缓存之间版本不一致。

### 为什么 Vite + Cargo 必须同一 bash 调用

MSYS2 bash 每次调用自动做 Unix/Windows 路径转换。分开调用 → 两次会话的路径转换结果可能不同 → PROTOC/LIBCLANG_PATH 值在 `C:/...` 和 `/c/...` 间波动 → prost-wkt-types、kcp-sys 的 build.rs 通过 `rerun-if-env-changed` 检测到变化 → easytier 依赖链全量重编。

同一 bash 调用内环境快照一致 → Cargo 指纹稳定 → 增量编译。

### 为什么 Gradle 必须用 PowerShell

bash (MSYS2) 文件操作与 Windows NTFS 文件锁冲突，Gradle transforms 阶段报 `Could not move temporary workspace`。PowerShell 使用 Windows 原生 API 无此问题。

### 为什么跳过 Gradle 的 rustBuildArm64Release

`rustBuildArm64Release` 内部调用 `pnpm tauri android android-studio-script`，依赖 `%TEMP%\com.kkrainbow.easytier-server-addr` 临时文件连接 Tauri CLI TCP 服务端。该服务端仅 Android Studio 启动时存在，命令行构建中不存在 → Tauri CLI panic。Step 1 已通过 `cargo build --lib` 生成了 `.so`，Step 2 手动复制并 `-x` 跳过该任务即可。

清理 `merged_jni_libs` / `merged_native_libs` / `stripped_native_libs` 确保 Gradle 不使用缓存的旧 `.so`。

### 指纹稳定性

Cargo 增量编译依赖"指纹"(fingerprint) 稳定。以下措施保证指纹不因无关因素失效：

1. **所有交叉编译 env var 固化在 `[env]`** — 不经过 shell export，Cargo 直接从配置文件读取，值恒定
2. **Vite + Cargo 同一 bash 调用** — 单次环境快照
3. **`GIT_CEILING_DIRECTORIES`** — easytier-core 的 `constants.rs` 使用 `git_version::git_version!()` proc-macro，编译时运行 `git describe` 并生成 `include_bytes!(".git/index")`。Cargo 基于 mtime 追踪该文件 → 任何 git 操作（含 IDE 后台刷新）都会更新 `.git/index` mtime → easytier-core 指纹失效。设置此变量后 proc-macro 找不到 git 仓库 → 走 fallback（`CARGO_PKG_VERSION`）→ 不生成 git 文件追踪 → 指纹稳定

开发构建的版本号因此显示纯 `CARGO_PKG_VERSION`（如 `2.7.0`），无 commit hash。CI 不受影响。

---

## 四、工具链

| 工具 | 路径 | 用途 |
|------|------|------|
| Rust | `$HOME/.cargo/bin` | 编译工具链 |
| MinGW-w64 | `$HOME/mingw64` | GCC 链接器（替代 VS Build Tools） |
| LLVM/libclang | `$HOME/llvm-dll`（`libclang.dll`） | kcp-sys bindgen |
| protoc | `$HOME/protoc/bin/protoc.exe` | prost-wkt-types |
| pnpm | `$HOME/.npm-global` | 前端包管理 |
| JDK 17 | `D:\tools\Java\jdk-17.0.12` | Android Gradle |
| Android SDK | `D:\tools\Android\Sdk`（NDK 27.0.12077973 + build-tools 34 + platform 34） | Android 编译 |

---

## 五、踩坑速查

| 问题 | 原因 | 解决 |
|------|------|------|
| 仅改前端却全量重编（~9min） | git-version proc-macro 追踪 `.git/index` mtime | `.cargo/config.toml` `[env]` 加 `GIT_CEILING_DIRECTORIES` |
| 跨会话 Cargo 全量重编 | MSYS2 路径转换导致 env var 值波动 | 全部 env var 固入 `[env]`，Vite+Cargo 同一 bash 调用 |
| Gradle `Could not move temporary workspace` | bash + NTFS 文件锁冲突 | 用 PowerShell，设 `$env:GRADLE_USER_HOME` |
| `link.exe` 找不到 | 无 VS Build Tools | 改用 MinGW-w64 |
| `libclang.dll` / `protoc` 找不到 | kcp-sys / prost-wkt-types 构建需要 | 安装 LLVM / protoc 到 `$HOME` |
| APK 不更新前端 | 前端嵌入 `.so`，且 Gradle 跳过 `rustBuild` 时不会同步 `assets/frontend/` | 前端变更必须跑 Cargo + Vite，同步资源并检查 APK 资源哈希 |
| `usesCleartextTraffic` 不生效 | Release 默认 false | Manifest 硬编码 + manifestPlaceholders |
| Windows 符号链接失败 | 非管理员无权限 | BuildTask.kt 改用文件复制 |
| Gradle 缓存损坏 | JDK 切换或磁盘问题 | 删 `~/.gradle/caches` |
