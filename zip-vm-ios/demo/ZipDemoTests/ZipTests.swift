import XCTest
@testable import ZipDemo

final class ZipTests: XCTestCase {
    func testVersion() {
        XCTAssertFalse(Zip.version.isEmpty)
        XCTAssertTrue(Zip.version.contains("7.2"))
    }

    func testEmptyProgram() {
        let result = Zip.compileAndRun("функция главная() { возврат(0) }")
        XCTAssertNotNil(result)
        XCTAssertTrue(result!.contains("0"), result ?? "")
    }

    func testArithmetic() {
        let result = Zip.compileAndRun("функция главная() { возврат(2 + 3) }")
        XCTAssertNotNil(result)
        XCTAssertTrue(result!.contains("5"), result ?? "")
    }

    func testFactorial() {
        let result = Zip.compileAndRun("""
            функция факториал(н) {
                если (н < 2) { возврат(1) }
                возврат(н * факториал(н - 1))
            }
            функция главная() { возврат(факториал(5)) }
            """)
        XCTAssertNotNil(result)
        XCTAssertTrue(result!.contains("120"), result ?? "")
    }

    func testString() {
        let result = Zip.compileAndRun("""
            функция главная() {
                напечатать_строку("Привет из iOS!")
                возврат(0)
            }
            """)
        XCTAssertNotNil(result)
        XCTAssertTrue(result!.contains("Привет"), result ?? "")
    }
}
