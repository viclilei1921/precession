package com.viclilei.precession.device

import android.app.Activity
import android.content.ClipData
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.util.Base64
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.core.content.FileProvider
import androidx.fragment.app.FragmentActivity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.io.FileInputStream
import java.io.IOException
import java.io.InputStream
import java.nio.ByteBuffer
import java.util.UUID
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

private const val ALIAS = "precession.device.v2"
private const val ANDROID_KEYSTORE = "AndroidKeyStore"
private const val GCM = "AES/GCM/NoPadding"
private const val GCM_TAG_BITS = 128

@InvokeArg
class EnrollArgs {
    lateinit var dekB64: String
}

@InvokeArg
class OpenArgs {
    lateinit var ctB64: String
}

@InvokeArg
class ShareArgs {
    lateinit var path: String
}

@InvokeArg
class CopyPickedArgs {
    lateinit var uri: String
}

@TauriPlugin
class DeviceSlotPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun enroll(invoke: Invoke) {
        if (!canConfirm()) {
            finish(invoke, false, "unavailable")
            return
        }
        val args = invoke.parseArgs(EnrollArgs::class.java)
        val dek = try {
            Base64.decode(args.dekB64, Base64.NO_WRAP)
        } catch (_: IllegalArgumentException) {
            finish(invoke, false, "internal")
            return
        }
        if (dek.size != 32) {
            dek.fill(0)
            finish(invoke, false, "internal")
            return
        }
        try {
            createKey()
            val cipher = Cipher.getInstance(GCM)
            cipher.init(Cipher.ENCRYPT_MODE, secretKey())
            val encrypted = cipher.doFinal(dek)
            val packed = pack(cipher.iv, encrypted)
            dek.fill(0)
            finish(invoke, true, ctB64 = Base64.encodeToString(packed, Base64.NO_WRAP))
        } catch (_: Exception) {
            dek.fill(0)
            deleteKey()
            finish(invoke, false, "unavailable")
        }
    }

    @Command
    fun open(invoke: Invoke) {
        if (!keystoreReady()) {
            finish(invoke, false, "unavailable")
            return
        }
        if (!hasKey()) {
            finish(invoke, false, "invalid")
            return
        }
        val args = invoke.parseArgs(OpenArgs::class.java)
        val packed = try {
            Base64.decode(args.ctB64, Base64.NO_WRAP)
        } catch (_: IllegalArgumentException) {
            finish(invoke, false, "invalid")
            return
        }
        val parts = unpack(packed)
        if (parts == null) {
            finish(invoke, false, "invalid")
            return
        }
        try {
            val cipher = Cipher.getInstance(GCM)
            cipher.init(Cipher.DECRYPT_MODE, secretKey(), GCMParameterSpec(GCM_TAG_BITS, parts.first))
            val dek = cipher.doFinal(parts.second)
            if (dek.size != 32) {
                dek.fill(0)
                finish(invoke, false, "invalid")
            } else {
                val dekB64 = Base64.encodeToString(dek, Base64.NO_WRAP)
                dek.fill(0)
                finish(invoke, true, dekB64 = dekB64)
            }
        } catch (_: KeyPermanentlyInvalidatedException) {
            finish(invoke, false, "invalid")
        } catch (_: Exception) {
            finish(invoke, false, "invalid")
        }
    }

    @Command
    fun confirm(invoke: Invoke) {
        val host = activity as? FragmentActivity
        if (host == null || !canConfirm()) {
            finish(invoke, false, "unavailable")
            return
        }
        val prompt = BiometricPrompt(
            host,
            ContextCompat.getMainExecutor(host),
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    finish(invoke, true)
                }

                override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                    val code = when (errorCode) {
                        BiometricPrompt.ERROR_USER_CANCELED,
                        BiometricPrompt.ERROR_NEGATIVE_BUTTON,
                        BiometricPrompt.ERROR_CANCELED,
                        -> "cancelled"
                        else -> "unavailable"
                    }
                    finish(invoke, false, code)
                }
            },
        )
        prompt.authenticate(promptInfo())
    }

    @Command
    fun share(invoke: Invoke) {
        val args = invoke.parseArgs(ShareArgs::class.java)
        val file = File(args.path)
        if (!file.isFile) {
            finish(invoke, false, "invalid")
            return
        }
        val uri = try {
            FileProvider.getUriForFile(activity, "${activity.packageName}.fileprovider", file)
        } catch (_: IllegalArgumentException) {
            finish(invoke, false, "invalid")
            return
        }
        val send = Intent(Intent.ACTION_SEND).apply {
            type = "application/zip"
            putExtra(Intent.EXTRA_STREAM, uri)
            clipData = ClipData.newRawUri("", uri)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
        try {
            activity.startActivity(Intent.createChooser(send, "导出档案"))
            finish(invoke, true)
        } catch (_: Exception) {
            finish(invoke, false, "unavailable")
        }
    }

    @Command
    fun copyPicked(invoke: Invoke) {
        val args = try {
            invoke.parseArgs(CopyPickedArgs::class.java)
        } catch (_: Exception) {
            finish(invoke, false, "invalid")
            return
        }
        Thread {
            val copied = try {
                copyIntoCache(args.uri)
            } catch (_: Exception) {
                null
            }
            activity.runOnUiThread {
                if (copied == null) {
                    finish(invoke, false, "invalid")
                } else {
                    finish(invoke, true, path = copied)
                }
            }
        }.start()
    }

    @Command
    fun forget(invoke: Invoke) {
        try {
            deleteKey()
            finish(invoke, true)
        } catch (_: Exception) {
            finish(invoke, false, "internal")
        }
    }

    private fun keystoreReady(): Boolean {
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.P
    }

    /** 启用和锁定后解锁共用：强生物识别或锁屏密码至少有一种可用。 */
    private fun canConfirm(): Boolean {
        if (!keystoreReady()) {
            return false
        }
        return BiometricManager.from(activity).canAuthenticate(authenticators()) ==
            BiometricManager.BIOMETRIC_SUCCESS
    }

    private fun authenticators(): Int {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            BiometricManager.Authenticators.BIOMETRIC_STRONG or
                BiometricManager.Authenticators.DEVICE_CREDENTIAL
        } else {
            BiometricManager.Authenticators.BIOMETRIC_STRONG
        }
    }

    private fun promptInfo(): BiometricPrompt.PromptInfo {
        val builder = BiometricPrompt.PromptInfo.Builder()
            .setTitle("解锁人生档案")
            .setSubtitle("通过系统验证以打开档案")
            .setAllowedAuthenticators(authenticators())
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) {
            builder.setNegativeButtonText("取消")
        }
        return builder.build()
    }

    private fun createKey() {
        deleteKey()
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        val builder = KeyGenParameterSpec.Builder(
            ALIAS,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setUnlockedDeviceRequired(true)
        generator.init(builder.build())
        generator.generateKey()
    }

    private fun secretKey(): SecretKey {
        val key = keyStore().getKey(ALIAS, null) ?: throw IllegalStateException("missing device key")
        return key as SecretKey
    }

    private fun hasKey(): Boolean {
        return try {
            keyStore().containsAlias(ALIAS)
        } catch (_: Exception) {
            false
        }
    }

    private fun deleteKey() {
        val keyStore = keyStore()
        if (keyStore.containsAlias(ALIAS)) {
            keyStore.deleteEntry(ALIAS)
        }
    }

    private fun keyStore(): KeyStore {
        return KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
    }

    private fun pack(iv: ByteArray, body: ByteArray): ByteArray {
        val buffer = ByteBuffer.allocate(4 + iv.size + body.size)
        buffer.putInt(iv.size)
        buffer.put(iv)
        buffer.put(body)
        return buffer.array()
    }

    private fun unpack(packed: ByteArray): Pair<ByteArray, ByteArray>? {
        if (packed.size < 5) {
            return null
        }
        val buffer = ByteBuffer.wrap(packed)
        val ivLen = buffer.int
        if (ivLen <= 0 || ivLen > buffer.remaining()) {
            return null
        }
        val iv = ByteArray(ivLen)
        buffer.get(iv)
        val body = ByteArray(buffer.remaining())
        buffer.get(body)
        if (body.isEmpty()) {
            return null
        }
        return iv to body
    }

    private fun copyIntoCache(raw: String): String {
        val uri = Uri.parse(raw)
        if (uri.scheme.isNullOrBlank()) {
            throw IllegalArgumentException("uri")
        }
        val dir = File(activity.cacheDir, "picked/${UUID.randomUUID()}")
        if (!dir.mkdirs() && !dir.isDirectory) {
            throw IOException("mkdir")
        }
        val dest = File(dir, safeName(displayName(uri)))
        val input = openStream(uri) ?: throw IOException("open")
        try {
            input.use { source ->
                dest.outputStream().use { output -> source.copyTo(output) }
            }
        } catch (err: Exception) {
            dest.delete()
            throw err
        }
        if (!dest.isFile) {
            throw IOException("missing")
        }
        return dest.absolutePath
    }

    private fun openStream(uri: Uri): InputStream? {
        try {
            activity.contentResolver.openInputStream(uri)?.let { return it }
        } catch (_: Exception) {
        }
        val docId = try {
            if (DocumentsContract.isDocumentUri(activity, uri)) {
                DocumentsContract.getDocumentId(uri)
            } else {
                null
            }
        } catch (_: Exception) {
            null
        }
        if (docId != null && docId.startsWith("raw:")) {
            val file = File(docId.removePrefix("raw:"))
            if (file.isFile) {
                return FileInputStream(file)
            }
        }
        return null
    }

    private fun displayName(uri: Uri): String {
        val queried = try {
            queryDisplayName(uri)
        } catch (_: Exception) {
            null
        }
        if (!queried.isNullOrBlank()) {
            return queried
        }
        val segment = Uri.decode(uri.lastPathSegment ?: return "book")
        val slash = segment.lastIndexOf('/')
        val tail = if (slash >= 0) segment.substring(slash + 1) else segment
        val colon = tail.lastIndexOf(':')
        val file = if (colon >= 0 && colon < tail.lastIndex) tail.substring(colon + 1) else tail
        return file.ifBlank { "book" }
    }

    private fun queryDisplayName(uri: Uri): String? {
        val cursor = activity.contentResolver.query(
            uri,
            arrayOf(OpenableColumns.DISPLAY_NAME),
            null,
            null,
            null,
        ) ?: return null
        cursor.use {
            if (!it.moveToFirst()) {
                return null
            }
            val index = it.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (index < 0) {
                return null
            }
            return it.getString(index)
        }
    }

    private fun safeName(raw: String): String {
        val cleaned = raw.replace(Regex("""[\\/\u0000]"""), "_").trim().trim('.')
        if (cleaned.isEmpty() || cleaned == "." || cleaned == "..") {
            return "book"
        }
        if (cleaned.length <= 180) {
            return cleaned
        }
        val dot = cleaned.lastIndexOf('.')
        if (dot > 0 && cleaned.length - dot <= 8) {
            val ext = cleaned.substring(dot)
            return cleaned.take(180 - ext.length).trimEnd('.') + ext
        }
        return cleaned.take(180)
    }

    private fun finish(
        invoke: Invoke,
        ok: Boolean,
        code: String? = null,
        ctB64: String? = null,
        dekB64: String? = null,
        path: String? = null,
    ) {
        val ret = JSObject()
        ret.put("ok", ok)
        if (code != null) {
            ret.put("code", code)
        }
        if (ctB64 != null) {
            ret.put("ctB64", ctB64)
        }
        if (dekB64 != null) {
            ret.put("dekB64", dekB64)
        }
        if (path != null) {
            ret.put("path", path)
        }
        invoke.resolve(ret)
    }
}
