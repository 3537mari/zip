# ZIP: A Cyrillic Programming Language That's 40× Faster Than Python

**TL;DR:** I built a programming language called ZIP (Zero-Immunity-Pulse) that uses only Cyrillic characters, compiles to native code via LLVM or custom bytecode, and benchmarks ~40× faster than Python and competitive with Rust/Go on real-world tasks (v2.1.0).

## Why another programming language?

I wanted a language that:

1. **Wasn't tied to Latin script** (linguistic sovereignty)
2. **Had semantic roots** (3-letter roots for concepts)
3. **Was fast** (native compilation + own VM)
4. **Had built-in AI primitives** (tensors, neural networks, agents)
5. **Was formally verified** (mathematical specs + `zip-verify`)
6. **Was self-hosted** (compiler written in itself, fixed point)

## What makes ZIP different?

### 1. Cyrillic-only syntax

```zip
функция сложить(а, б) {
    возврат(а + б)
}
```

Every keyword and identifier is Cyrillic. The language has:

- **64 semantic roots** (СПР = count, ЧТВ = memory, ЖВТ = life)
- **10 type modifiers via vowels** (А-А = verb, И-А = buffer, О-О = tool)
- **Complementary roots** (СПР ↔ РПС for reverse operations)

### 2. Flow-oriented paradigm (VSO)

Instead of objects-first, ZIP emphasizes flows:

```zip
САПАР(данные) >> ЧАТАВ!
```

- `>>` — impulse (move semantics, no copying)
- `||` — parallel choir
- `!` — PERFECT lock (immutable)
- `~` — async stream

### 3. AI-native

```zip
#зависимость("нейросети = 1.0")

слой1 = создать_слой(2, 4)
слой2 = создать_слой(4, 1)
выход = прямой_проход_слоя(слой2, прямой_проход_слоя(слой1, вход))
```

Built-in / STD primitives for tensors, neural nets, agents, and streams.

### 4. Formally verified

- Semantics specification (invariants)
- Theorems sketched/proven for type safety, ownership, PERFECT
- `zip-verify` for static analysis

### 5. Self-hosted path

Compiler written in ZIP; ZIPB bytecode + VM v3; roadmap to full independence from Rust/C (v3–v4).

## Benchmarks (v2.1.0)

| Task | ZIP | Python | Rust | Go |
|------|-----|--------|------|-----|
| Fibonacci(40) | 1395 ms | 17883 ms | 336 ms | 723 ms |
| GEMM 512² | 24 ms | 3.5 ms (NumPy) | 216 ms* | 646 ms* |
| JSON 10k | 5 ms | 77 ms | 6.5 ms | 23 ms |
| Strings 1M | 64 ms | 1392 ms | 106 ms | 86 ms |
| HTTP parse 5M | 45 ms | 6519 ms | 42 ms | 56 ms |
| XOR 10k epochs | 7 ms | 333 ms | 8.7 ms | 5.8 ms |

\* naive triple-loop in Rust/Go; ZIP GEMM uses blocked C/clang STD runtime.  
Average: ZIP ~**40×** faster than CPython; often within ~1–2× of Rust depending on the task.

Full write-up: [docs/БЕНЧМАРКИ.md](https://github.com/3537mari/zip/blob/master/docs/БЕНЧМАРКИ.md)

## Real-world example: TODO REST service

Production demo with auth, JSON file DB, CRUD:

- [examples/todo_сервис](https://github.com/3537mari/zip/tree/master/examples/todo_сервис)
- [docs/ПРИМЕР-ВЕБ-СЕРВИС.md](https://github.com/3537mari/zip/blob/master/docs/ПРИМЕР-ВЕБ-СЕРВИС.md)

## What's next?

- **v3.0.0** (2027): C bootstrap, optional LLVM
- **v4.0.0**: no C-runtime, self-host on ZIP
- **v5.0.0**: cross-compilation (ARM, RISC-V, WASM)

## Try it

```bash
git clone https://github.com/3537mari/zip.git
cd zip
cargo build --release --manifest-path zipc/Cargo.toml
./zipc/target/release/zipc запустить examples/привет.zip
```

## Links

- GitHub: https://github.com/3537mari/zip
- Website: https://3537mari.github.io/zip/
- Docs: https://github.com/3537mari/zip/tree/master/docs
- Release v2.1.0: https://github.com/3537mari/zip/releases/tag/v2.1.0

I'd love feedback from the HN community!
