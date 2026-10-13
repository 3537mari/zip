import SwiftUI

struct MainView: View {
    @State private var source: String = """
    функция главная() {
        напечатать_строку("Привет из iOS!")
        возврат(42)
    }
    """
    @State private var output: String = "Нажмите «Запустить»"
    @State private var isLoading: Bool = false

    var body: some View {
        NavigationView {
            VStack(alignment: .leading, spacing: 12) {
                Text("ZIP v\(Zip.version)")
                    .font(.caption)
                    .foregroundColor(.gray)

                TextEditor(text: $source)
                    .font(.system(.body, design: .monospaced))
                    .frame(height: 200)
                    .border(Color.gray.opacity(0.3))

                HStack {
                    Button("Скомпилировать") {
                        if let hex = Zip.compile(source) {
                            let preview = hex.prefix(64)
                            let bytes = hex.count / 2
                            output = "ZIPB: \(preview)… (\(bytes) байт)"
                        } else {
                            output = "Ошибка компиляции"
                        }
                    }

                    Button("Запустить") {
                        isLoading = true
                        DispatchQueue.global(qos: .userInitiated).async {
                            let result = ZipRunner.run(source)
                            DispatchQueue.main.async {
                                output = result.output
                                isLoading = false
                            }
                        }
                    }
                    .disabled(isLoading)

                    if isLoading {
                        ProgressView()
                    }
                }

                ScrollView {
                    Text(output)
                        .font(.system(.body, design: .monospaced))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding()
                        .background(Color.gray.opacity(0.1))
                        .cornerRadius(8)
                }

                Spacer()
            }
            .padding()
            .navigationTitle("ZIP Demo")
        }
    }
}
