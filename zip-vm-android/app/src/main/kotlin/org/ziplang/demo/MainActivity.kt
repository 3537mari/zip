package org.ziplang.demo

import android.os.Bundle
import android.widget.Button
import android.widget.EditText
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import org.ziplang.zip.Zip

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val editText = findViewById<EditText>(R.id.source)
        val button = findViewById<Button>(R.id.run)
        val result = findViewById<TextView>(R.id.result)

        result.text = "ZIP v${Zip.версия()}"

        button.setOnClickListener {
            val code = editText.text.toString()
            result.text = try {
                Zip.скомпилироватьИЗапустить(code)
            } catch (e: Throwable) {
                "Ошибка: ${e.message}"
            }
        }
    }
}
