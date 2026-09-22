package com.viclilei.precession.device

import android.app.Activity
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.util.Base64
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.nio.ByteBuffer
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

@TauriPlugin
class DeviceSlotPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun enroll(invoke: Invoke) {
        if (!supported()) {
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
        if (!supported()) {
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
        if (host == null) {
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
        val info = BiometricPrompt.PromptInfo.Builder()
            .setTitle("解锁人生档案")
            .setSubtitle("验证生物识别以打开档案")
            .setNegativeButtonText("取消")
            .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
            .build()
        prompt.authenticate(info)
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

    private fun supported(): Boolean {
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.P
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

    private fun finish(
        invoke: Invoke,
        ok: Boolean,
        code: String? = null,
        ctB64: String? = null,
        dekB64: String? = null,
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
        invoke.resolve(ret)
    }
}
