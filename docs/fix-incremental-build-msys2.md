# 增量编译缓存失效修复：MSYS2 路径转换导致 Cargo 指纹不稳定

## 现象

仅修改前端代码（home.vue），Cargo 构建每次都是全量重编译（~7m45s），而非预期的增量（~1m25s）。easytier-core 每次都被重新编译。

## 根因

`.cargo/config.toml` 中 CC/AR/BINDGEN 等交叉编译变量已固化在 `[env]` 块，指纹稳定。但 PROTOC 和 LIBCLANG_PATH 仍在 bash 中通过 `export` 注入：

```bash
export PROTOC="C:/Users/lymly/protoc/bin/protoc.exe"
export LIBCLANG_PATH="C:/Users/lymly/llvm-dll"
```

MSYS2 bash 每次调用时自动对路径值做 Unix/Windows 格式转换，导致 Cargo 进程看到的实际值在 `C:/Users/...` 和 `/c/Users/...` 之间波动（取决于 MSYS2 的路径转换启发式策略）。`prost-wkt-types` 和 `kcp-sys` 等依赖 crate 的 build.rs 通过 `rerun-if-env-changed` 追踪这些变量，每次值微变就触发依赖链全量重编。

**受影响的依赖链**：
```
LIBCLANG_PATH 不稳定 → kcp-sys (bindgen) 重编译 → easytier 重编译
PROTOC 不稳定 → prost-wkt-types 重编译 → easytier 重编译
```

## 解决方案

将所有交叉编译环境变量统一放入 `.cargo/config.toml` 的 `[env]` 块：

```toml
[env]
CC_aarch64_linux_android = "D:/tools/Android/Sdk/ndk/..."
AR_aarch64_linux_android = "D:/tools/Android/Sdk/ndk/..."
BINDGEN_EXTRA_CLANG_ARGS_aarch64_linux_android = "..."
PROTOC = "C:/Users/lymly/protoc/bin/protoc.exe"
LIBCLANG_PATH = "C:/Users/lymly/llvm-dll"
```

`[env]` 中的值由 Cargo 直接从配置文件读取，不经过 shell 环境变量的 MSYS2 路径转换层，因此每次构建的值完全一致，指纹稳定。

## 效果

| 场景 | 修复前 | 修复后 |
|------|--------|--------|
| 前端变更 | ~9m20s (Vite 53s + Cargo 7m45s + Gradle 42s) | **~2m30s** (Vite 53s + Cargo 1m25s + Gradle 11s) |
| 无变更 | ~9min | **~1m06s** (Vite 53s + Cargo 1.3s + Gradle 11s) |

Cargo 输出中 easytier 不再出现 "Compiling" 字样（仅 warnings），确认缓存命中。

## 教训

- **永远不要在 MSYS2 bash 中用 export 设置 Rust 构建依赖的 env var**——用 `.cargo/config.toml` 的 `[env]` 代替。
- MSYS2 路径转换是静默且不可控的，即使每次写了完全相同的路径字符串也可能出现不同的转换结果。
- 诊断方法：连续两次 `cargo build` 观察输出，若没有源码变更但出现 "Compiling" → 指纹失效 → 检查 `config.toml` 和 shell env var 差异。
