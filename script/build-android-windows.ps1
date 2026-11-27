[CmdletBinding()]
param(
  [string]$JdkHome = 'D:\tools\Java\jdk-17.0.12',
  [string]$AndroidHome = 'D:\tools\Android\Sdk',
  [string]$NdkVersion = '27.0.12077973',
  [string]$MinGwRoot = 'C:\Users\lymly\mingw64\mingw64',
  [string]$CargoBin = 'C:\Users\lymly\.cargo\bin',
  [string]$LlvmDllDir = 'C:\Users\lymly\llvm-dll',
  [string]$ProtocExe = 'C:\Users\lymly\protoc\bin\protoc.exe',
  [string]$RustToolchain = 'stable-x86_64-pc-windows-gnu',
  [string]$OutputDir = 'F:\tmp\apk',
  [switch]$InstallDeps,
  [switch]$BuildFrontendDeps,
  [switch]$SkipCargo,
  [switch]$SkipGradle,
  [switch]$CleanAndroid,
  [switch]$ValidateOnly
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$guiDir = Join-Path $root 'easytier-gui'
$tauriDir = Join-Path $guiDir 'src-tauri'
$androidDir = Join-Path $tauriDir 'gen\android'
$ndkHome = Join-Path $AndroidHome "ndk\$NdkVersion"
$ndkBin = Join-Path $ndkHome 'toolchains\llvm\prebuilt\windows-x86_64\bin'
$mingwBin = Join-Path $MinGwRoot 'bin'
$rustup = Join-Path $CargoBin 'rustup.exe'
$cargo = Join-Path $CargoBin 'cargo.exe'
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

function Require-Directory([string]$Path, [string]$Label) {
  if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
    throw "$Label not found: $Path"
  }
}

function Require-File([string]$Path, [string]$Label) {
  if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "$Label not found: $Path"
  }
}

function Assert-LastExitCode([string]$Step) {
  if ($LASTEXITCODE -ne 0) {
    throw "$Step failed with exit code $LASTEXITCODE"
  }
}

function Invoke-Pnpm([string[]]$Arguments) {
  $corepack = (Get-Command corepack -ErrorAction Stop).Source
  $env:CI = 'true'
  & $corepack pnpm@9.12.1 @Arguments
  Assert-LastExitCode "pnpm $($Arguments -join ' ')"
}

Require-Directory $root 'Repository root'
Require-Directory $JdkHome 'JDK 17'
Require-Directory $AndroidHome 'Android SDK'
Require-Directory $ndkHome "Android NDK $NdkVersion"
Require-Directory $ndkBin 'Android NDK LLVM bin'
Require-Directory $mingwBin 'MinGW bin'
Require-Directory $LlvmDllDir 'libclang directory'
Require-File $rustup 'rustup'
Require-File $cargo 'cargo'
Require-File $ProtocExe 'protoc'
Require-File (Join-Path $JdkHome 'bin\java.exe') 'Java'
Require-File (Join-Path $ndkBin 'aarch64-linux-android24-clang.cmd') 'Android arm64 Clang'
Require-File (Join-Path $ndkBin 'llvm-ar.exe') 'Android llvm-ar'
Require-File (Join-Path $mingwBin 'gcc.exe') 'MinGW gcc'

$env:JAVA_HOME = $JdkHome
$env:ANDROID_HOME = $AndroidHome
$env:ANDROID_SDK_ROOT = $AndroidHome
$env:NDK_HOME = $ndkHome
$env:ANDROID_NDK_HOME = $ndkHome
$env:LIBCLANG_PATH = $LlvmDllDir
$env:PROTOC = $ProtocExe
$env:GIT_CEILING_DIRECTORIES = $root
$env:GRADLE_USER_HOME = 'D:\tmp\gradle-tmp'
$env:CC_aarch64_linux_android = Join-Path $ndkBin 'aarch64-linux-android24-clang.cmd'
$env:AR_aarch64_linux_android = Join-Path $ndkBin 'llvm-ar.exe'
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $env:CC_aarch64_linux_android
$env:Path = "$JdkHome\bin;$ndkBin;$mingwBin;$CargoBin;$env:Path"

$installedTargets = & $rustup run $RustToolchain rustup target list --installed
Assert-LastExitCode 'rustup target list'
if ($installedTargets -notcontains 'aarch64-linux-android') {
  Write-Host "Installing aarch64-linux-android for $RustToolchain..."
  $env:RUSTUP_DIST_SERVER = 'https://rsproxy.cn'
  $env:RUSTUP_UPDATE_ROOT = 'https://rsproxy.cn/rustup'
  & $rustup target add --toolchain $RustToolchain aarch64-linux-android
  Assert-LastExitCode 'rustup target add'
}

Write-Host "Repository: $root"
Write-Host "Rust toolchain: $RustToolchain"
Write-Host "Android NDK: $NdkVersion"
Write-Host "JDK: $JdkHome"

$cargoHome = Split-Path -Parent $CargoBin
$lockText = Get-Content -Raw (Join-Path $root 'Cargo.lock')
$registryRoots = @(Get-ChildItem -LiteralPath (Join-Path $cargoHome 'registry\src') -Directory)
if ($registryRoots.Count -eq 0) {
  throw "Cargo registry source directory not found under $cargoHome\registry\src"
}

function Get-LockedPackageVersion([string]$Name) {
  $escaped = [regex]::Escape($Name)
  $pattern = "(?ms)^\[\[package\]\]\s*\r?\nname = `"$escaped`"\s*\r?\nversion = `"([^`"]+)`""
  $match = [regex]::Match($lockText, $pattern)
  if (-not $match.Success) {
    throw "Package not found in Cargo.lock: $Name"
  }
  return $match.Groups[1].Value
}

function Resolve-RegistryPackageDir([string]$Name) {
  $version = Get-LockedPackageVersion $Name
  $directoryName = "$Name-$version"
  foreach ($registryRoot in $registryRoots) {
    $candidate = Join-Path $registryRoot.FullName $directoryName
    if (Test-Path -LiteralPath $candidate -PathType Container) {
      return $candidate
    }
  }
  throw "Cargo registry source not found: $directoryName"
}

$settingsSpecs = @(
  @{ Project = ':tauri-android'; Package = 'tauri'; Relative = 'mobile/android' },
  @{ Project = ':tauri-plugin-clipboard-manager'; Package = 'tauri-plugin-clipboard-manager'; Relative = 'android' },
  @{ Project = ':tauri-plugin-shell'; Package = 'tauri-plugin-shell'; Relative = 'android' },
  @{ Project = ':tauri-plugin-vpnservice'; Package = 'tauri-plugin-vpnservice'; Relative = 'android' },
  @{ Project = ':tauri-plugin-opener'; Package = 'tauri-plugin-opener'; Relative = 'android' }
)

$settingsLines = @('// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.')
foreach ($spec in $settingsSpecs) {
  $packageDir = if ($spec.Package -eq 'tauri-plugin-vpnservice') {
    Join-Path $root 'tauri-plugin-vpnservice'
  }
  else {
    Resolve-RegistryPackageDir $spec.Package
  }
  $projectDir = (Join-Path $packageDir $spec.Relative).Replace('\', '/')
  Require-Directory $projectDir "$($spec.Project) Android project"
  $settingsLines += "include '$($spec.Project)'"
  $settingsLines += "project('$($spec.Project)').projectDir = new File(`"$projectDir`")"
}
$settingsPath = Join-Path $androidDir 'tauri.settings.gradle'
$settingsLines | Set-Content -LiteralPath $settingsPath -Encoding utf8NoBOM
Write-Host "Generated Windows Gradle settings: $settingsPath"

if ($ValidateOnly) {
  Write-Host 'Validation completed successfully.'
  $stopwatch.Stop()
  Write-Host "Elapsed: $([math]::Round($stopwatch.Elapsed.TotalSeconds, 1))s"
  exit 0
}

if ($InstallDeps -or -not (Test-Path -LiteralPath (Join-Path $root 'node_modules'))) {
  Push-Location $guiDir
  try {
    Invoke-Pnpm -Arguments @('install', '--frozen-lockfile', '--ignore-scripts')
  }
  finally {
    Pop-Location
  }
}

if ($BuildFrontendDeps) {
  Push-Location $root
  try {
    Invoke-Pnpm -Arguments @('--dir', 'tauri-plugin-vpnservice', 'build')
    Invoke-Pnpm -Arguments @('--dir', 'easytier-web/frontend-lib', 'build')
  }
  finally {
    Pop-Location
  }
}

Push-Location $guiDir
try {
  Invoke-Pnpm -Arguments @('run', 'build')
}
finally {
  Pop-Location
}

$dist = Join-Path $guiDir 'dist'
Require-File (Join-Path $dist 'index.html') 'Frontend build output'

if (-not $SkipCargo) {
  $cargoArguments = @(
    'build',
    '--manifest-path', (Join-Path $tauriDir 'Cargo.toml'),
    '--package', 'easytier-gui',
    '--lib',
    '--release',
    '--target', 'aarch64-linux-android',
    '--features', 'tauri/custom-protocol',
    '--locked'
  )
  Push-Location $root
  try {
    & $rustup run $RustToolchain cargo @cargoArguments
    Assert-LastExitCode 'cargo build'
  }
  finally {
    Pop-Location
  }
}

$soPath = Join-Path $root 'target\aarch64-linux-android\release\libapp_lib.so'
Require-File $soPath 'Android arm64 native library'
$jniPath = Join-Path $androidDir 'app\src\main\jniLibs\arm64-v8a\libapp_lib.so'
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $jniPath) | Out-Null
Copy-Item -LiteralPath $soPath -Destination $jniPath -Force

$assetsPath = Join-Path $androidDir 'app\src\main\assets\frontend'
$resolvedAssets = [System.IO.Path]::GetFullPath($assetsPath)
$resolvedAndroid = [System.IO.Path]::GetFullPath($androidDir)
if (-not $resolvedAssets.StartsWith($resolvedAndroid + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Unexpected frontend assets path: $resolvedAssets"
}
if (Test-Path -LiteralPath $assetsPath) {
  Remove-Item -LiteralPath $assetsPath -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $assetsPath | Out-Null
Copy-Item -Path (Join-Path $dist '*') -Destination $assetsPath -Recurse -Force

foreach ($name in @('merged_jni_libs', 'merged_native_libs', 'stripped_native_libs')) {
  $path = Join-Path $androidDir "app\build\intermediates\$name"
  $resolved = [System.IO.Path]::GetFullPath($path)
  if (-not $resolved.StartsWith($resolvedAndroid + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Unexpected Gradle intermediate path: $resolved"
  }
  if (Test-Path -LiteralPath $path) {
    Remove-Item -LiteralPath $path -Recurse -Force
  }
}

if (-not $SkipGradle) {
  Push-Location $androidDir
  try {
    $gradleTasks = if ($CleanAndroid) { @(':app:clean', 'assembleArm64Release') } else { @('assembleArm64Release') }
    $gradleCommand = ".\gradlew.bat $($gradleTasks -join ' ') --no-daemon --exclude-task=rustBuildArm64Release"
    & cmd.exe /d /c $gradleCommand
    Assert-LastExitCode 'Gradle assembleArm64Release'
  }
  finally {
    Pop-Location
  }
}

$apkPath = Join-Path $androidDir 'app\build\outputs\apk\arm64\release\app-arm64-release.apk'
Require-File $apkPath 'Built APK'
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$timestamp = Get-Date -Format 'yyyyMMddHHmm'
$outputApk = Join-Path $OutputDir "app-arm64-release-$timestamp.apk"
Copy-Item -LiteralPath $apkPath -Destination $outputApk -Force

$apksigner = Join-Path $AndroidHome 'build-tools\34.0.0\apksigner.bat'
Require-File $apksigner 'apksigner'
& $apksigner verify --verbose $outputApk
Assert-LastExitCode 'apksigner verify'

Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead($outputApk)
try {
  $nativeEntry = $zip.GetEntry('lib/arm64-v8a/libapp_lib.so')
  if (-not $nativeEntry) {
    throw 'APK does not contain lib/arm64-v8a/libapp_lib.so'
  }
  $stream = $nativeEntry.Open()
  try {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
      $apkNativeHash = ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-', '')
    }
    finally {
      $sha.Dispose()
    }
  }
  finally {
    $stream.Dispose()
  }
  $sourceNativeHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $soPath).Hash
  if ($apkNativeHash -ne $sourceNativeHash) {
    throw 'APK native library hash does not match the Cargo output'
  }

  $frontendEntries = @($zip.Entries | Where-Object { $_.FullName -like 'assets/frontend/*' -and $_.FullName -notlike '*/' })
  if ($frontendEntries.Count -eq 0) {
    throw 'APK does not contain frontend assets'
  }
  $mismatches = @()
  foreach ($entry in $frontendEntries) {
    $relative = $entry.FullName.Substring('assets/frontend/'.Length)
    $local = Join-Path $dist ($relative.Replace('/', '\'))
    if (-not (Test-Path -LiteralPath $local)) {
      $mismatches += "missing local file: $relative"
      continue
    }
    $stream = $entry.Open()
    try {
      $sha = [System.Security.Cryptography.SHA256]::Create()
      try {
        $entryHash = ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-', '')
      }
      finally {
        $sha.Dispose()
      }
    }
    finally {
      $stream.Dispose()
    }
    if ($entryHash -ne (Get-FileHash -Algorithm SHA256 -LiteralPath $local).Hash) {
      $mismatches += "hash mismatch: $relative"
    }
  }
  if ($mismatches.Count -gt 0) {
    throw "Frontend asset verification failed: $($mismatches -join ', ')"
  }
}
finally {
  $zip.Dispose()
}

$outputHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $outputApk).Hash
$file = Get-Item -LiteralPath $outputApk
$stopwatch.Stop()
Write-Host ''
Write-Host 'Android arm64 release build completed.'
Write-Host "APK: $($file.FullName)"
Write-Host "Size: $($file.Length) bytes"
Write-Host "SHA256: $outputHash"
Write-Host "Elapsed: $([math]::Round($stopwatch.Elapsed.TotalSeconds, 1))s"
