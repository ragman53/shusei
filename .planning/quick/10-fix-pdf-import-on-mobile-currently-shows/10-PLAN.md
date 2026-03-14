---
phase: quick-10
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - platform/android/app/src/main/java/com/shusei/app/MainActivity.java
  - src/platform/android.rs
  - src/ui/library.rs
autonomous: true
requirements: [QUICK-10]
user_setup: []

must_haves:
  truths:
    - "User can tap Import PDF button on Android device"
    - "System file picker opens allowing PDF selection"
    - "Selected PDF is imported and appears in library"
  artifacts:
    - path: "platform/android/app/src/main/java/com/shusei/app/MainActivity.java"
      provides: "Native file picker via Intent.ACTION_OPEN_DOCUMENT"
      contains: "pickPdfFile"
    - path: "src/platform/android.rs"
      provides: "JNI bridge for file picker"
      exports: ["pick_file implementation"]
    - path: "src/ui/library.rs"
      provides: "UI handler using platform pick_file"
      contains: "handle_import_pdf uses pick_file"
  key_links:
    - from: "src/ui/library.rs"
      to: "platform::get_platform_api().pick_file"
      via: "async call"
    - from: "src/platform/android.rs"
      to: "MainActivity.pickPdfFile"
      via: "JNI static method call"
---

<objective>
Enable PDF import on Android by implementing native file picker functionality.

Purpose: Users currently see "PDF import not available on mobile" when trying to import PDFs on Android. This needs to work properly using Android's native file picker.

Output: Working PDF import flow on Android devices.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution>

<context>
@.planning/STATE.md
@src/ui/library.rs
@src/platform/android.rs
@src/platform/mod.rs
@platform/android/app/src/main/java/com/shusei/app/MainActivity.java

<interfaces>
<!-- Existing patterns from codebase -->

From src/platform/mod.rs:
```rust
#[async_trait]
pub trait PlatformApi: Send + Sync {
    async fn pick_file(&self, extensions: &[&str]) -> Result<String>;
    // ... other methods
}
```

From src/platform/android.rs (current stub):
```rust
async fn pick_file(&self, _extensions: &[&str]) -> Result<String> {
    Err(ShuseiError::Platform(
        "File picker not yet implemented on Android.".into()
    ).into())
}
```

From MainActivity.java (existing pattern for static JNI methods):
```java
public static void startCameraCapture() {
    if (instance == null) {
        Log.e(TAG, "startCameraCapture: instance is null");
        notifyCaptureFailed("Activity instance not available");
        return;
    }
    instance.runOnUiThread(() -> {
        // implementation
    });
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>task 1: Add Android file picker JNI method in MainActivity.java</name>
  <files>platform/android/app/src/main/java/com/shusei/app/MainActivity.java</files>
  <action>
    Add file picker functionality using Android's Storage Access Framework (Intent.ACTION_OPEN_DOCUMENT):
    
    1. Add a static method `pickPdfFile()` that:
       - Creates an Intent with `Intent.ACTION_OPEN_DOCUMENT`
       - Sets type to `"application/pdf"`
       - Adds category `Intent.CATEGORY_OPENABLE`
       - Starts activity with a request code (e.g., 1002)
    
    2. Add `onActivityResult` override to handle the picked file:
       - Extract URI from result intent
       - Copy file content to app's files directory (since SAF URIs are temporary)
       - Call native method `onFilePicked(String path)` to notify Rust
    
    3. Add native method declaration:
       - `private native void onFilePicked(String filePath);`
       - `private native void onFilePickFailed(String errorMessage);`
    
    4. Add fields for pending picker callback:
       - `private static final int FILE_PICKER_REQUEST = 1002;`
    
    Pattern to follow (similar to camera capture):
    - Use singleton instance pattern
    - Run on UI thread
    - Handle null instance gracefully
  </action>
  <verify>
    <automated>grep -q "pickPdfFile\|ACTION_OPEN_DOCUMENT\|onFilePicked" platform/android/app/src/main/java/com/shusei/app/MainActivity.java</automated>
  </verify>
  <done>
    - pickPdfFile() static method exists and launches file picker intent
    - onActivityResult handles picked file and calls native method
    - Native method declarations added for callbacks
  </done>
</task>

<task type="auto">
  <name>task 2: Implement Rust JNI bridge for file picker in android.rs</name>
  <files>src/platform/android.rs</files>
  <action>
    Implement the `pick_file` method for AndroidPlatform:
    
    1. Add static storage for pending file picker result (similar to CAMERA_STATE):
       ```rust
       static FILE_PICKER_STATE: Mutex<Option<FilePickerState>> = Mutex::new(None);
       
       struct FilePickerState {
           result_sender: Option<oneshot::Sender<Result<String>>>,
       }
       ```
    
    2. Implement `pick_file` in the PlatformApi impl:
       - Create oneshot channel
       - Store sender in FILE_PICKER_STATE
       - Call Java method via JNI: call `MainActivity.pickPdfFile()`
       - Await result with timeout (60 seconds for user to pick file)
    
    3. Add JNI callback functions:
       ```rust
       #[no_mangle]
       pub extern "system" fn Java_com_shusei_app_MainActivity_onFilePicked(
           mut env: JNIEnv,
           _class: JClass,
           file_path: jni::sys::jstring,
       ) {
           // Convert jstring to Rust String
           // Send success result through FILE_PICKER_STATE
       }
       
       #[no_mangle]
       pub extern "system" fn Java_com_shusei_app_MainActivity_onFilePickFailed(
           mut env: JNIEnv,
           _class: JClass,
           error_message: jni::sys::jstring,
       ) {
           // Send error result through FILE_PICKER_STATE
       }
       ```
    
    Pattern to follow: Same as camera capture implementation (lines 68-99).
  </action>
  <verify>
    <automated>grep -q "FILE_PICKER_STATE\|onFilePicked\|pick_file" src/platform/android.rs</automated>
  </verify>
  <done>
    - FILE_PICKER_STATE static added
    - pick_file method implemented with JNI call and async await
    - JNI callback functions onFilePicked and onFilePickFailed added
  </done>
</task>

<task type="auto">
  <name>task 3: Update library.rs to use platform pick_file on Android</name>
  <files>src/ui/library.rs</files>
  <action>
    Replace the Android stub `handle_import_pdf` with actual file picker integration:
    
    Current code (lines 174-179):
    ```rust
    #[cfg(target_os = "android")]
    let handle_import_pdf = move |_| {
        spawn(async move {
            error_message.set(Some("PDF import not available on mobile".to_string()));
        });
    };
    ```
    
    New implementation should:
    1. Use `crate::platform::get_platform_api().pick_file(&["pdf"]).await` to open file picker
    2. If file picked successfully, use the returned path with PdfProcessor.import_pdf()
    3. Show metadata review dialog (same flow as desktop)
    4. Handle errors gracefully
    
    Follow the same pattern as the desktop implementation (lines 117-172) but use `pick_file` instead of `rfd::AsyncFileDialog`.
  </action>
  <verify>
    <automated>grep -q "pick_file\|PDF import not available" src/ui/library.rs && ! grep -q "PDF import not available on mobile" src/ui/library.rs</automated>
  </verify>
  <done>
    - Android handle_import_pdf uses platform pick_file API
    - Same import flow as desktop (metadata review, database save)
    - Error message removed, replaced with working implementation
  </done>
</task>

</tasks>

<verification>
1. Build compiles without errors: `cargo build --target aarch64-linux-android`
2. No "PDF import not available" message in codebase
3. All three files have the new implementation
</verification>

<success_criteria>
- User can tap "Import PDF" on Android device
- Native file picker opens showing PDF files
- Selecting a PDF imports it into the library
- PDF appears in library list after import
</success_criteria>

<output>
After completion, create `.planning/quick/10-fix-pdf-import-on-mobile-currently-shows/10-SUMMARY.md`
</output>