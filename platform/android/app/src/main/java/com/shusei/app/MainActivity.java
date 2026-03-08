package com.shusei.app;

import android.Manifest;
import android.annotation.SuppressLint;
import android.content.Context;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.ImageFormat;
import android.graphics.Rect;
import android.graphics.YuvImage;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCaptureSession;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraDevice;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.CaptureRequest;
import android.hardware.camera2.params.StreamConfigurationMap;
import android.media.Image;
import android.media.ImageReader;
import android.os.Bundle;
import android.os.Handler;
import android.os.HandlerThread;
import android.util.Log;
import android.util.Size;
import android.view.Surface;

import androidx.annotation.NonNull;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import java.io.ByteArrayOutputStream;
import java.nio.ByteBuffer;
import java.util.Arrays;
import java.util.concurrent.Semaphore;
import java.util.concurrent.TimeUnit;

/**
 * MainActivity with Camera2 API support for the Shusei app.
 * This class bridges Rust JNI calls to Android Camera2 API.
 */
public class MainActivity extends androidx.appcompat.app.AppCompatActivity {
    private static final String TAG = "ShuseiCamera";
    private static final int CAMERA_PERMISSION_REQUEST = 1001;
    
    // Singleton instance for JNI access
    private static MainActivity instance;
    
    // Camera API objects
    private CameraDevice cameraDevice;
    private CameraCaptureSession captureSession;
    private ImageReader imageReader;
    private CameraManager cameraManager;
    private HandlerThread backgroundThread;
    private Handler backgroundHandler;
    
    // Camera state
    private String cameraId;
    private Size imageDimension;
    private Semaphore cameraLock = new Semaphore(1);
    private boolean isCapturing = false;
    
    // Load native library
    static {
        System.loadLibrary("shusei");
    }
    
    // Native methods
    private native void nativeInit();
    private native void onImageCaptured(byte[] imageData, int width, int height);
    private native void onImageCaptureFailed(String errorMessage);
    private native void onAudioRecorded(float[] audioData, int sampleRate);
    private native void onPermissionResult(String permission, boolean granted);
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Store instance for JNI access
        instance = this;
        
        // Initialize camera manager
        cameraManager = (CameraManager) getSystemService(Context.CAMERA_SERVICE);
        
        // Initialize native code
        try {
            nativeInit();
            Log.i(TAG, "Native library initialized successfully");
        } catch (UnsatisfiedLinkError e) {
            Log.e(TAG, "Failed to load native library", e);
        }
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        startBackgroundThread();
    }
    
    @Override
    protected void onPause() {
        stopBackgroundThread();
        closeCamera();
        super.onPause();
    }
    
    @Override
    protected void onDestroy() {
        instance = null;
        super.onDestroy();
    }
    
    // ==================== Static JNI Methods ====================
    
    /**
     * Check if camera permission is granted.
     * Called from Rust via JNI.
     */
    public static boolean hasCameraPermission() {
        if (instance == null) {
            Log.w(TAG, "hasCameraPermission: instance is null");
            return false;
        }
        return ContextCompat.checkSelfPermission(instance, Manifest.permission.CAMERA)
                == PackageManager.PERMISSION_GRANTED;
    }
    
    /**
     * Request camera permission.
     * Called from Rust via JNI.
     */
    public static void requestCameraPermission() {
        if (instance == null) {
            Log.w(TAG, "requestCameraPermission: instance is null");
            return;
        }
        
        if (ContextCompat.checkSelfPermission(instance, Manifest.permission.CAMERA)
                != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(instance,
                    new String[]{Manifest.permission.CAMERA},
                    CAMERA_PERMISSION_REQUEST);
        } else {
            instance.onPermissionResult(Manifest.permission.CAMERA, true);
        }
    }
    
    /**
     * Start camera capture.
     * Called from Rust via JNI.
     */
    public static void startCameraCapture() {
        if (instance == null) {
            Log.e(TAG, "startCameraCapture: instance is null");
            // Use the static native method
            notifyCaptureFailed("Activity instance not available");
            return;
        }
        
        instance.runOnUiThread(() -> {
            instance.openCameraAndCapture();
        });
    }
    
    /**
     * Vibrate the device.
     * Called from Rust via JNI.
     */
    public static void vibrate(long durationMs) {
        if (instance == null) {
            Log.w(TAG, "vibrate: instance is null");
            return;
        }
        
        android.os.Vibrator vibrator = (android.os.Vibrator) instance.getSystemService(Context.VIBRATOR_SERVICE);
        if (vibrator != null && vibrator.hasVibrator()) {
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
                vibrator.vibrate(android.os.VibrationEffect.createOneShot(
                        durationMs, android.os.VibrationEffect.DEFAULT_AMPLITUDE));
            } else {
                vibrator.vibrate(durationMs);
            }
        }
    }
    
    // Static method to notify capture failure when instance is null
    private static native void notifyCaptureFailed(String errorMessage);
    
    // ==================== Permission Handling ====================
    
    @Override
    public void onRequestPermissionsResult(int requestCode, @NonNull String[] permissions,
                                           @NonNull int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        
        if (requestCode == CAMERA_PERMISSION_REQUEST) {
            boolean granted = grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED;
            onPermissionResult(Manifest.permission.CAMERA, granted);
        }
    }
    
    // ==================== Camera2 API ====================
    
    private void startBackgroundThread() {
        backgroundThread = new HandlerThread("CameraBackground");
        backgroundThread.start();
        backgroundHandler = new Handler(backgroundThread.getLooper());
    }
    
    private void stopBackgroundThread() {
        if (backgroundThread != null) {
            backgroundThread.quitSafely();
            try {
                backgroundThread.join();
                backgroundThread = null;
                backgroundHandler = null;
            } catch (InterruptedException e) {
                Log.e(TAG, "Error stopping background thread", e);
            }
        }
    }
    
    /**
     * Open the camera and start capture.
     */
    @SuppressLint("MissingPermission")
    private void openCameraAndCapture() {
        if (isCapturing) {
            Log.w(TAG, "Camera capture already in progress");
            return;
        }
        
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA)
                != PackageManager.PERMISSION_GRANTED) {
            onImageCaptureFailed("Camera permission not granted");
            return;
        }
        
        try {
            // Get back camera ID
            for (String id : cameraManager.getCameraIdList()) {
                CameraCharacteristics characteristics = cameraManager.getCameraCharacteristics(id);
                Integer facing = characteristics.get(CameraCharacteristics.LENS_FACING);
                if (facing != null && facing == CameraCharacteristics.LENS_FACING_BACK) {
                    cameraId = id;
                    
                    StreamConfigurationMap map = characteristics.get(
                            CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
                    if (map != null) {
                        imageDimension = chooseOptimalSize(map.getOutputSizes(ImageFormat.JPEG), 640, 480);
                    }
                    break;
                }
            }
            
            if (cameraId == null) {
                onImageCaptureFailed("No back camera found");
                return;
            }
            
            // Create ImageReader for capturing JPEG images
            imageReader = ImageReader.newInstance(
                    imageDimension.getWidth(),
                    imageDimension.getHeight(),
                    ImageFormat.JPEG,
                    1
            );
            imageReader.setOnImageAvailableListener(onImageAvailableListener, backgroundHandler);
            
            // Open camera
            isCapturing = true;
            cameraManager.openCamera(cameraId, stateCallback, backgroundHandler);
            
        } catch (CameraAccessException e) {
            Log.e(TAG, "Failed to open camera", e);
            onImageCaptureFailed("Failed to open camera: " + e.getMessage());
            isCapturing = false;
        }
    }
    
    private final CameraDevice.StateCallback stateCallback = new CameraDevice.StateCallback() {
        @Override
        public void onOpened(@NonNull CameraDevice camera) {
            cameraDevice = camera;
            createCaptureSession();
        }
        
        @Override
        public void onDisconnected(@NonNull CameraDevice camera) {
            camera.close();
            cameraDevice = null;
            isCapturing = false;
        }
        
        @Override
        public void onError(@NonNull CameraDevice camera, int error) {
            camera.close();
            cameraDevice = null;
            isCapturing = false;
            onImageCaptureFailed("Camera device error: " + error);
        }
    };
    
    private void createCaptureSession() {
        try {
            cameraDevice.createCaptureSession(
                    Arrays.asList(imageReader.getSurface()),
                    captureSessionStateCallback,
                    backgroundHandler
            );
        } catch (CameraAccessException e) {
            Log.e(TAG, "Failed to create capture session", e);
            onImageCaptureFailed("Failed to create capture session: " + e.getMessage());
            isCapturing = false;
        }
    }
    
    private final CameraCaptureSession.StateCallback captureSessionStateCallback =
            new CameraCaptureSession.StateCallback() {
        @Override
        public void onConfigured(@NonNull CameraCaptureSession session) {
            captureSession = session;
            captureStillImage();
        }
        
        @Override
        public void onConfigureFailed(@NonNull CameraCaptureSession session) {
            onImageCaptureFailed("Failed to configure camera session");
            isCapturing = false;
        }
    };
    
    private void captureStillImage() {
        try {
            CaptureRequest.Builder captureBuilder = cameraDevice.createCaptureRequest(
                    CameraDevice.TEMPLATE_STILL_CAPTURE);
            captureBuilder.addTarget(imageReader.getSurface());
            
            // Set capture parameters
            captureBuilder.set(CaptureRequest.CONTROL_MODE, CaptureRequest.CONTROL_MODE_AUTO);
            
            captureSession.capture(captureBuilder.build(), captureCallback, backgroundHandler);
        } catch (CameraAccessException e) {
            Log.e(TAG, "Failed to capture image", e);
            onImageCaptureFailed("Failed to capture image: " + e.getMessage());
            isCapturing = false;
        }
    }
    
    private final CameraCaptureSession.CaptureCallback captureCallback =
            new CameraCaptureSession.CaptureCallback() {
        @Override
        public void onCaptureCompleted(@NonNull CameraCaptureSession session,
                                       @NonNull CaptureRequest request,
                                       @NonNull android.hardware.camera2.TotalCaptureResult result) {
            Log.i(TAG, "Image capture completed");
        }
        
        @Override
        public void onCaptureFailed(@NonNull CameraCaptureSession session,
                                    @NonNull CaptureRequest request,
                                    @NonNull android.hardware.camera2.CaptureFailure failure) {
            Log.e(TAG, "Image capture failed: " + failure.getReason());
            onImageCaptureFailed("Capture failed: " + failure.getReason());
            isCapturing = false;
        }
    };
    
    private final ImageReader.OnImageAvailableListener onImageAvailableListener =
            reader -> {
        Image image = null;
        try {
            image = reader.acquireLatestImage();
            if (image != null) {
                byte[] jpegData = imageToByteArray(image);
                int width = imageDimension.getWidth();
                int height = imageDimension.getHeight();
                
                Log.i(TAG, "Image captured: " + jpegData.length + " bytes, " + width + "x" + height);
                
                // Send to Rust via JNI
                onImageCaptured(jpegData, width, height);
                
                image.close();
            }
        } catch (Exception e) {
            Log.e(TAG, "Error processing captured image", e);
            onImageCaptureFailed("Error processing image: " + e.getMessage());
        } finally {
            if (image != null) {
                image.close();
            }
            closeCamera();
            isCapturing = false;
        }
    };
    
    private byte[] imageToByteArray(Image image) {
        ByteBuffer buffer = image.getPlanes()[0].getBuffer();
        byte[] data = new byte[buffer.remaining()];
        buffer.get(data);
        return data;
    }
    
    private void closeCamera() {
        if (captureSession != null) {
            captureSession.close();
            captureSession = null;
        }
        if (cameraDevice != null) {
            cameraDevice.close();
            cameraDevice = null;
        }
        if (imageReader != null) {
            imageReader.close();
            imageReader = null;
        }
    }
    
    private Size chooseOptimalSize(Size[] choices, int targetWidth, int targetHeight) {
        for (Size size : choices) {
            if (size.getWidth() >= targetWidth && size.getHeight() >= targetHeight) {
                return size;
            }
        }
        // Return the largest available if no match
        return choices[0];
    }
}