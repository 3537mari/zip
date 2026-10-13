// Zip.swift
// ZIP-VM для iOS — обёртка над C-API (zip.h).

import Foundation

/// ZIP-VM для iOS.
public enum Zip {

    /// Компилировать ZIP-исходник в ZIPB (hex-строка).
    public static func compile(_ source: String) -> String? {
        guard let resultPtr = source.withCString({ zip_compile($0) }) else {
            return nil
        }
        defer { zip_free(resultPtr) }
        return String(cString: resultPtr)
    }

    /// Запустить ZIPB (hex-строка), вернуть вывод.
    public static func run(_ bytecodeHex: String) -> String? {
        guard let resultPtr = bytecodeHex.withCString({ zip_run($0) }) else {
            return nil
        }
        defer { zip_free(resultPtr) }
        return String(cString: resultPtr)
    }

    /// Полная цепочка: исходник → результат.
    public static func compileAndRun(_ source: String) -> String? {
        guard let resultPtr = source.withCString({ zip_compile_and_run($0) }) else {
            return nil
        }
        defer { zip_free(resultPtr) }
        return String(cString: resultPtr)
    }

    /// Версия ZIP.
    public static var version: String {
        String(cString: zip_version())
    }

    /// Успех последней C-операции.
    public static var lastSuccess: Bool {
        zip_last_success() != 0
    }
}
