package dev.dioxus.main

import android.os.Bundle
import androidx.activity.OnBackPressedCallback
import io.github.taled.editor.BuildConfig
import java.io.File
import java.io.PrintWriter
import java.io.StringWriter

typealias BuildConfig = BuildConfig

class MainActivity : WryActivity() {
    private external fun nativeOnBackPressed()

    override fun onCreate(savedInstanceState: Bundle?) {
        installCrashLogger()
        appendBootstrapLog("activity:onCreate:start")
        try {
            super.onCreate(savedInstanceState)
            onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    appendBootstrapLog("activity:onBackPressedDispatcher")
                    nativeOnBackPressed()
                }
            })
            appendBootstrapLog("activity:onCreate:ok")
        } catch (throwable: Throwable) {
            appendBootstrapLog("activity:onCreate:throw\n${stackTrace(throwable)}")
            throw throwable
        }
    }

    override fun onStart() {
        appendBootstrapLog("activity:onStart")
        super.onStart()
    }

    override fun onResume() {
        appendBootstrapLog("activity:onResume")
        super.onResume()
    }

    override fun onPause() {
        appendBootstrapLog("activity:onPause")
        super.onPause()
    }

    override fun onStop() {
        appendBootstrapLog("activity:onStop")
        super.onStop()
    }

    override fun onDestroy() {
        appendBootstrapLog("activity:onDestroy")
        super.onDestroy()
    }

    private fun installCrashLogger() {
        val previous = Thread.getDefaultUncaughtExceptionHandler()
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            appendBootstrapLog("uncaught:${thread.name}\n${stackTrace(throwable)}")
            previous?.uncaughtException(thread, throwable)
        }
    }

    private fun appendBootstrapLog(message: String) {
        try {
            val dir = getExternalFilesDir("logs") ?: File(filesDir, "logs")
            if (!dir.exists()) {
                dir.mkdirs()
            }
            File(dir, "taled-editor.log").appendText(
                "[${System.currentTimeMillis()}] java: $message\n"
            )
        } catch (_: Throwable) {
        }
    }

    private fun stackTrace(throwable: Throwable): String {
        val writer = StringWriter()
        throwable.printStackTrace(PrintWriter(writer))
        return writer.toString()
    }
}
