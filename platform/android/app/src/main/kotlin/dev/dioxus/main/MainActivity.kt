package dev.dioxus.main

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.util.Log
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageCapture
import androidx.camera.core.ImageCaptureException
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.lifecycle.LifecycleOwner
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.IOException
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

/**
 * MainActivity with CameraX implementation for the Shusei app.
 * This class bridges Rust JNI calls to Android CameraX API.
 */
class MainActivity : WryActivity() {
    companion object {
        private const val TAG = "ShuseiCamera"
        private const val TAG_FILE = "ShuseiFile"
        private const val CAMERA_PERMISSION_REQUEST = 1001
        private val REQUIRED_PERMISSIONS = arrayOf(Manifest.permission.CAMERA)
        
        // Singleton instance for JNI access
        @Volatile
        private var instance: MainActivity? = null
        
        // CameraX executor (shared)
        private val cameraExecutor: ExecutorService = Executors.newSingleThreadExecutor()
        
        /**
         * Check if camera permission is granted.
         * Called from Rust via JNI.
         */
        @JvmStatic
        fun hasCameraPermission(): Boolean {
            val ctx = instance ?: run {
                Log.w(TAG, "hasCameraPermission: instance is null")
                return false
            }
            return REQUIRED_PERMISSIONS.all {
                ContextCompat.checkSelfPermission(ctx, it) == PackageManager.PERMISSION_GRANTED
            }
        }
        
        /**
         * Request camera permission.
         * Called from Rust via JNI.
         */
        @JvmStatic
        fun requestCameraPermission() {
            val ctx = instance ?: run {
                Log.w(TAG, "requestCameraPermission: instance is null")
                return
            }
            
            if (REQUIRED_PERMISSIONS.any {
                    ContextCompat.checkSelfPermission(ctx, it) != PackageManager.PERMISSION_GRANTED
                }) {
                ActivityCompat.requestPermissions(ctx, REQUIRED_PERMISSIONS, CAMERA_PERMISSION_REQUEST)
            } else {
                onPermissionResult(Manifest.permission.CAMERA, true)
            }
        }
        
        /**
         * Start camera capture using CameraX.
         * Called from Rust via JNI.
         */
        @JvmStatic
        fun startCameraCapture() {
            Log.i(TAG, "startCameraCapture called")
            
            val ctx = instance ?: run {
                Log.e(TAG, "startCameraCapture: instance is null")
                notifyCaptureFailed("Activity instance not available")
                return
            }
            
            // Check permission first
            if (!hasCameraPermission()) {
                Log.w(TAG, "startCameraCapture: permission not granted, requesting...")
                requestCameraPermission()
                return
            }
            
            ctx.runOnUiThread {
                ctx.bindCameraUseCases()
            }
        }
        
        /**
         * Launch PDF file picker using Storage Access Framework.
         * Called from Rust via JNI.
         */
        @JvmStatic
        fun pickPdfFile() {
            Log.i(TAG_FILE, "pickPdfFile called")
            
            val ctx = instance ?: run {
                Log.e(TAG_FILE, "pickPdfFile: instance is null")
                onFilePickFailed("Activity instance not available")
                return
            }
            
            val launcher = ctx.filePickerLauncher ?: run {
                Log.e(TAG_FILE, "pickPdfFile: filePickerLauncher not initialized")
                onFilePickFailed("File picker not initialized")
                return
            }
            
            ctx.runOnUiThread {
                Log.i(TAG_FILE, "Launching file picker for PDF")
                launcher.launch(arrayOf("application/pdf"))
            }
        }
        
        /**
         * Vibrate the device.
         * Called from Rust via JNI.
         */
        @JvmStatic
        fun vibrate(durationMs: Long) {
            Log.d(TAG, "vibrate: ${durationMs}ms")
            
            val ctx = instance ?: run {
                Log.w(TAG, "vibrate: instance is null")
                return
            }
            
            val vibrator = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val vibratorManager = ctx.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager
                vibratorManager.defaultVibrator
            } else {
                @Suppress("DEPRECATION")
                ctx.getSystemService(VIBRATOR_SERVICE) as Vibrator
            }
            
            if (vibrator.hasVibrator()) {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    vibrator.vibrate(VibrationEffect.createOneShot(durationMs, VibrationEffect.DEFAULT_AMPLITUDE))
                } else {
                    @Suppress("DEPRECATION")
                    vibrator.vibrate(durationMs)
                }
            }
        }
        
        /**
         * Copy a file from a content URI to internal storage.
         * Returns the absolute path of the copied file, or null on failure.
         */
        @JvmStatic
        fun copyUriToFiles(context: Context, uri: Uri): String? {
            val timestamp = System.currentTimeMillis()
            val fileName = "picked_${timestamp}.pdf"
            val outputFile = File(context.filesDir, fileName)
            
            Log.i(TAG_FILE, "copyUriToFiles: copying $uri to ${outputFile.absolutePath}")
            
            try {
                context.contentResolver.openInputStream(uri)?.use { inputStream ->
                    outputFile.outputStream().use { outputStream ->
                        inputStream.copyTo(outputStream)
                    }
                }
                
                Log.i(TAG_FILE, "File copied successfully: ${outputFile.absolutePath}")
                return outputFile.absolutePath
            } catch (e: IOException) {
                Log.e(TAG_FILE, "Failed to copy file: ${e.message}")
                return null
            }
        }
        
        // Native methods (callbacks to Rust)
        @JvmStatic
        private external fun onImageCaptured(imageData: ByteArray, width: Int, height: Int)
        
        @JvmStatic
        private external fun onImageCaptureFailed(errorMessage: String)
        
        @JvmStatic
        internal external fun onPermissionResult(permission: String, granted: Boolean)
        
        @JvmStatic
        internal external fun notifyCaptureFailed(errorMessage: String)
        
        @JvmStatic
        private external fun onFilePicked(filePath: String)
        
        @JvmStatic
        private external fun onFilePickFailed(errorMessage: String)
    }
    
    // Instance-level components
    private var imageCapture: ImageCapture? = null
    private var filePickerLauncher: ActivityResultLauncher<Array<String>>? = null
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Store instance for JNI access
        instance = this
        Log.i(TAG, "MainActivity created, instance stored")
        
        // Initialize file picker launcher
        filePickerLauncher = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
            Log.i(TAG_FILE, "File picker result: $uri")
            
            if (uri == null) {
                Log.w(TAG_FILE, "File picker cancelled by user")
                onFilePickFailed("User cancelled")
                return@registerForActivityResult
            }
            
            Log.i(TAG_FILE, "File selected, copying to internal storage...")
            try {
                val path = copyUriToFiles(this, uri)
                if (path != null) {
                    Log.i(TAG_FILE, "File copied successfully: $path")
                    onFilePicked(path)
                } else {
                    Log.e(TAG_FILE, "Failed to copy file")
                    onFilePickFailed("Failed to copy file")
                }
            } catch (e: IOException) {
                Log.e(TAG_FILE, "IO error copying file: ${e.message}")
                onFilePickFailed("IO error: ${e.message}")
            }
        }
        Log.i(TAG_FILE, "File picker launcher initialized")
    }
    
    override fun onDestroy() {
        super.onDestroy()
        instance = null
        cameraExecutor.shutdown()
        Log.i(TAG, "MainActivity destroyed")
    }
    
    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        
        if (requestCode == CAMERA_PERMISSION_REQUEST) {
            val allGranted = grantResults.all { it == PackageManager.PERMISSION_GRANTED }
            Log.i(TAG, "Permission result: $allGranted")
            onPermissionResult(Manifest.permission.CAMERA, allGranted)
            
            if (allGranted) {
                // Permission granted, start capture
                startCameraCapture()
            } else {
                notifyCaptureFailed("Camera permission denied")
            }
        }
    }
    
    /**
     * Bind CameraX use cases to the camera provider.
     */
    private fun bindCameraUseCases() {
        Log.i(TAG, "bindCameraUseCases called")
        
        val cameraProviderFuture = ProcessCameraProvider.getInstance(this)
        
        cameraProviderFuture.addListener({
            val cameraProvider = cameraProviderFuture.get()
            
            // Preview use case
            val preview = Preview.Builder()
                .build()
                .also {
                    it.setSurfaceProvider(PreviewView(this).surfaceProvider)
                }
            
            // ImageCapture use case
            imageCapture = ImageCapture.Builder()
                .setCaptureMode(ImageCapture.CAPTURE_MODE_MAXIMIZE_QUALITY)
                .setTargetRotation(windowManager.defaultDisplay.rotation)
                .build()
            
            // Camera selector (back camera)
            val cameraSelector = CameraSelector.DEFAULT_BACK_CAMERA
            
            try {
                // Unbind all use cases before rebinding
                cameraProvider.unbindAll()
                
                // Bind use cases to camera
                cameraProvider.bindToLifecycle(
                    this as LifecycleOwner,
                    cameraSelector,
                    preview,
                    imageCapture
                )
                
                Log.i(TAG, "CameraX use cases bound successfully")
                
                // Take the picture after a short delay to ensure camera is ready
                window.decorView.postDelayed({
                    takePhoto()
                }, 500)
                
            } catch (e: Exception) {
                Log.e(TAG, "Failed to bind camera use cases", e)
                onImageCaptureFailed("Failed to initialize camera: ${e.message}")
            }
        }, ContextCompat.getMainExecutor(this))
    }
    
    /**
     * Capture a photo using CameraX ImageCapture.
     */
    private fun takePhoto() {
        Log.i(TAG, "takePhoto called")
        
        val imageCapture = imageCapture ?: run {
            Log.e(TAG, "takePhoto: imageCapture is null")
            onImageCaptureFailed("Camera not initialized")
            return
        }
        
        // Create output options for in-memory capture
        val outputStream = ByteArrayOutputStream()
        val outputOptions = ImageCapture.OutputFileOptions.Builder(outputStream).build()
        
        imageCapture.takePicture(
            outputOptions,
            ContextCompat.getMainExecutor(this),
            object : ImageCapture.OnImageSavedCallback {
                override fun onImageSaved(output: ImageCapture.OutputFileResults) {
                    Log.i(TAG, "onImageSaved called")
                    
                    val imageData = outputStream.toByteArray()
                    
                    // Get image dimensions - use typical Full HD dimensions
                    // In a production app, you'd extract this from EXIF or the ImageProxy
                    val width = 1920  // Typical Full HD width
                    val height = 1080 // Typical Full HD height
                    
                    Log.i(TAG, "Image captured: ${imageData.size} bytes, ${width}x${height}")
                    
                    // Send to Rust via JNI
                    onImageCaptured(imageData, width, height)
                }
                
                override fun onError(exception: ImageCaptureException) {
                    Log.e(TAG, "Image capture failed", exception)
                    onImageCaptureFailed("Capture failed: ${exception.message}")
                }
            }
        )
    }
}
