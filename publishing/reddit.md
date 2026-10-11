# Reddit posts

## r/programming

**Title:** ZIP — Cyrillic programming language, ~40× faster than Python, self-hosted path

**Body:**

I released **ZIP v2.1.0** — a Cyrillic-only, flow-oriented language with native compilation (LLVM / own ZIPB VM), AI STD primitives, formal verification tooling, and a working TODO REST API demo.

**Why it might interest you**

- Cyrillic syntax + 64 semantic roots + vowel type modifiers  
- Move-style impulse operator `>>`, PERFECT locks `!`, parallel `||`  
- Benchmarks vs Python / Rust / Go (see table below)  
- Compiler self-hosting story + roadmap to drop Rust/C bootstrap  

**Benchmarks (highlights)**

| Task | ZIP | Python | Rust | Go |
|------|-----|--------|------|-----|
| Fib(40) | 1395 ms | 17883 ms | 336 ms | 723 ms |
| JSON 10k | 5 ms | 77 ms | 6.5 ms | 23 ms |
| Strings 1M | 64 ms | 1392 ms | 106 ms | 86 ms |
| XOR 10k | 7 ms | 333 ms | 8.7 ms | 5.8 ms |

Details & methodology: https://github.com/3537mari/zip/blob/master/docs/БЕНЧМАРКИ.md  

**Repo:** https://github.com/3537mari/zip  
**Site:** https://3537mari.github.io/zip/  
**Release:** https://github.com/3537mari/zip/releases/tag/v2.1.0  

Happy to answer questions / take criticism.

---

## r/rust

**Title:** Built a language with Rust-inspired move semantics + Cyrillic syntax — competitive with Rust on some benches

**Body:**

Hi r/rust,

I've been working on **ZIP**, a language that borrows ideas from Rust's ownership story and applies them to a **flow-oriented (VSO)** model.

**Rust-inspired**

- Move via `>>` (impulse)  
- Borrow / ownership analysis in the toolchain  
- Comptime, specialization, range Copy, match helpers  

**ZIP-specific**

- Cyrillic-only surface syntax  
- 64 semantic roots + vowel-based type tags  
- AI STD (tensors / XOR training demos)  
- Post-quantum crypto in STD  
- Formal semantics docs + `zip-verify`  

**Honest benchmark note:** on Fibonacci, Rust still wins; on JSON/strings ZIP can beat or match Rust; GEMM depends on BLAS vs naive loops. Full numbers: https://github.com/3537mari/zip/blob/master/docs/БЕНЧМАРКИ.md  

Compiler path is moving toward self-host (ZIP → ZIPB → VM); current bootstrap still uses Rust.

Repo: https://github.com/3537mari/zip  

Would love Rustacean feedback — especially on the ownership / PERFECT model.

---

## r/compsci

**Title:** Formalizing a flow-oriented language (ZIP): semantics, ownership, PERFECT invariant

**Body:**

I've been documenting formal aspects of **ZIP** — a Cyrillic, flow-oriented language (impulse / choir / PERFECT lock).

**Focus areas**

1. Type safety for operations in the IR / VM  
2. Ownership / move along impulse edges (no aliasing after move)  
3. PERFECT invariant — locked values stay immutable  
4. Self-compilation fixed-point as an engineering check  

Docs:

- https://github.com/3537mari/zip/blob/master/docs/ФОРМАЛЬНАЯ-СЕМАНТИКА.md  
- https://github.com/3537mari/zip/blob/master/docs/ТЕОРЕМЫ.md  
- https://github.com/3537mari/zip/blob/master/docs/ВЕРИФИКАЦИЯ.md  

Tool: `zip-verify`  

I'd welcome discussion on the formalization (what's underspecified, better proof structure, related work).

---

## r/Python

**Title:** Built a language ~40× faster than CPython on our suite — Python still wins on ecosystem

**Body:**

Not a "Python is dead" post — CPython's ecosystem is unmatched. Sharing benchmarks from **ZIP**, a Cyrillic language with native/LLVM and custom VM backends.

On our suite (fib, JSON parse, string churn, HTTP parse microbench, XOR train), ZIP averaged ~**40×** faster than CPython 3.12. NumPy still wins GEMM (BLAS).

Methodology + caveats: https://github.com/3537mari/zip/blob/master/docs/БЕНЧМАРКИ.md  
Repo: https://github.com/3537mari/zip  

Curious what the community thinks is the fair way to compare "language" vs "stdlib/runtime" performance.
