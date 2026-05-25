import java.io.File
import org.apache.tools.ant.taskdefs.condition.Os
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.logging.LogLevel
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction

open class BuildTask : DefaultTask() {
    @Input
    var rootDirRel: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null

    @TaskAction
    fun assemble() {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")

        val rustTarget = when (target) {
            "aarch64" -> "aarch64-linux-android"
            "armv7" -> "armv7-linux-androideabi"
            "i686" -> "i686-linux-android"
            "x86_64" -> "x86_64-linux-android"
            else -> throw GradleException("Unknown target: $target")
        }
        val jniLibArch = when (target) {
            "aarch64" -> "arm64-v8a"
            "armv7" -> "armeabi-v7a"
            "i686" -> "x86"
            "x86_64" -> "x86_64"
            else -> throw GradleException("Unknown target: $target")
        }

        val srcSo = File("D:/projects/projectsAlpha/EasyTier/target/$rustTarget/release/libapp_lib.so")
        val dstDir = File(project.projectDir, "src/main/jniLibs/$jniLibArch")
        val dstSo = File(dstDir, "libapp_lib.so")

        System.err.println("BuildTask($target): src=$srcSo exists=${srcSo.exists()}")

        if (srcSo.exists()) {
            dstDir.mkdirs()
            dstSo.delete()
            srcSo.copyTo(dstSo, overwrite = true)
            System.err.println("BuildTask($target): copied .so to $dstSo")
        } else {
            System.err.println("BuildTask($target): .so not found at $srcSo")
            // Try the original Tauri build as fallback
            val pnpmPath = File(System.getProperty("user.home"), ".npm-global/pnpm.cmd")
            val exe = if (pnpmPath.exists()) pnpmPath.absolutePath else "pnpm.cmd"
            System.err.println("BuildTask($target): running Tauri build with $exe")
            project.exec {
                workingDir = File(project.projectDir, rootDirRel)
                executable(exe)
                args(listOf("tauri", "android", "android-studio-script"))
                if (release) {
                    args("--release")
                }
                args(listOf("--target", target))
            }.assertNormalExitValue()
        }
    }
}
