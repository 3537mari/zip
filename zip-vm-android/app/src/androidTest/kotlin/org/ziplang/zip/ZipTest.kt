package org.ziplang.zip

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ZipTest {
    @Test
    fun тест_пустая_программа() {
        val вывод = Zip.скомпилироватьИЗапустить("функция главная() { возврат(0) }")
        assertTrue(вывод.contains("0"))
    }

    @Test
    fun тест_арифметика() {
        val вывод = Zip.скомпилироватьИЗапустить("функция главная() { возврат(2 + 3) }")
        assertTrue(вывод.contains("5"))
    }

    @Test
    fun тест_факториал() {
        val вывод = Zip.скомпилироватьИЗапустить(
            """
            функция факториал(н) {
                если (н < 2) { возврат(1) }
                возврат(н * факториал(н - 1))
            }
            функция главная() { возврат(факториал(5)) }
            """.trimIndent()
        )
        assertTrue(вывод.contains("120"))
    }

    @Test
    fun тест_версия() {
        assertTrue(Zip.версия().isNotEmpty())
    }
}
