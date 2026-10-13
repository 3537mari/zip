// ZipResult.swift
import Foundation

/// Результат работы ZIP.
public struct ZipResult {
    public let success: Bool
    public let output: String

    public init(success: Bool, output: String) {
        self.success = success
        self.output = output
    }
}

/// Высокоуровневый API для UI.
public enum ZipRunner {
    public static func run(_ source: String) -> ZipResult {
        if let result = Zip.compileAndRun(source) {
            return ZipResult(success: true, output: result)
        } else {
            return ZipResult(success: false, output: "Ошибка: не удалось запустить")
        }
    }
}
