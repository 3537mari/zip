package org.ziplang.zip

/**
 * ZIP-VM для Android.
 *
 * ```
 * val результат = Zip.скомпилироватьИЗапустить("""
 *     функция главная() {
 *         напечатать_строку("Привет из Android!")
 *         возврат(42)
 *     }
 * """)
 * ```
 */
object Zip {
    init {
        System.loadLibrary("zip_vm_android")
    }

    /** Компилировать ZIP → ZIPB (hex). */
    fun скомпилировать(исходник: String): String = compile(исходник)

    /** Запустить ZIPB (hex) → вывод. */
    fun запустить(байткодHex: String): String = run(байткодHex)

    /** Исходник → вывод. */
    fun скомпилироватьИЗапустить(исходник: String): String = compileAndRun(исходник)

    /** Версия биндинга. */
    fun версия(): String = version()

    @JvmStatic private external fun compile(source: String): String
    @JvmStatic private external fun run(bytecodeHex: String): String
    @JvmStatic private external fun compileAndRun(source: String): String
    @JvmStatic private external fun version(): String
}
