# ZIP-VM для Android (NDK + JNI)

Нативная библиотека `libzip_vm_android.so` + Kotlin API `org.ziplang.zip.Zip`.

## Сборка `.so`

```bash
# NDK + cargo-ndk
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk

export ANDROID_NDK_HOME=...   # путь к NDK
cargo ndk -t arm64-v8a -t x86_64 \
  -o ./app/src/main/jniLibs \
  build --release
```

## APK

```bash
./gradlew assembleDebug
```

Подробнее: [docs/ANDROID.md](../docs/ANDROID.md).
