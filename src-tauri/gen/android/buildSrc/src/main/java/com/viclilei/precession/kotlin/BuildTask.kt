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
        val executable = """pnpm""";
        try {
            runTauriCli(executable)
        } catch (e: Exception) {
            if (Os.isFamily(Os.FAMILY_WINDOWS)) {
                // Try different Windows-specific extensions
                val fallbacks = listOf(
                    "$executable.exe",
                    "$executable.cmd",
                    "$executable.bat",
                )
                
                var lastException: Exception = e
                for (fallback in fallbacks) {
                    try {
                        runTauriCli(fallback)
                        return
                    } catch (fallbackException: Exception) {
                        lastException = fallbackException
                    }
                }
                throw lastException
            } else {
                throw e;
            }
        }
    }

    fun runTauriCli(executable: String) {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")
        val cliArgs = mutableListOf("tauri", "android", "android-studio-script")

        if (project.logger.isEnabled(LogLevel.DEBUG)) {
            cliArgs.add("-vv")
        } else if (project.logger.isEnabled(LogLevel.INFO)) {
            cliArgs.add("-v")
        }
        if (release) {
            cliArgs.add("--release")
        }
        cliArgs.addAll(listOf("--target", target))

        project.exec {
            workingDir(File(project.projectDir, rootDirRel))
            if (Os.isFamily(Os.FAMILY_WINDOWS)) {
                executable(executable)
                args(cliArgs)
            } else {
                // The Gradle daemon resolves executables with the PATH it had at
                // startup. fnm/corepack's `pnpm` is a per-shell symlink, so a
                // daemon left over from an earlier shell fails with
                // "A problem occurred starting process 'command pnpm'".
                // /bin/sh is a stable binary; the shell then finds pnpm on the
                // PATH of this build. `tauri android init` overwrites this file.
                executable("/bin/sh")
                args("-c")
                args("exec " + (listOf(executable) + cliArgs).joinToString(" ") { shellSingleQuote(it) })
            }
        }.assertNormalExitValue()
    }

    private fun shellSingleQuote(value: String): String =
        "'" + value.replace("'", "'\\''") + "'"
}