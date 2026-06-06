# 本地 Android 构建

EasyTier Tauri v2 Android App 本地（Windows）构建指南。

> CI 全平台构建请参阅 [docs/CI全平台构建.md](CI全平台构建.md)。

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

```powershell
Copy-Item -Force "D:\projects\projectsAlpha\EasyTier\easytier-gui\src-tauri\gen\android\app\build\outputs\apk\arm64\release\app-arm64-release.apk" "F:\tmp\app-arm64-release.apk"
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

Tauri 的 `tauri_build::build()` 在 Cargo 编译阶段将 `dist/` 以 BROTLI 压缩嵌入 `.so`。**即使只改前端，也必须重新编译 .so**，否则 APK 加载旧前端。

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

开发构建的版本号因此显示纯 `CARGO_PKG_VERSION`（如 `2.6.4`），无 commit hash。CI 不受影响。

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
| APK 不更新前端 | Tauri 将 `dist/` 嵌入 `.so` | 前端变更也必须跑 Cargo |
| `usesCleartextTraffic` 不生效 | Release 默认 false | Manifest 硬编码 + manifestPlaceholders |
| Windows 符号链接失败 | 非管理员无权限 | BuildTask.kt 改用文件复制 |
| Gradle 缓存损坏 | JDK 切换或磁盘问题 | 删 `~/.gradle/caches` |
