//! Конвертация байт-кода ZIP → LLVM IR для JIT (Идея 5 — Миелинизация).
//!
//! Модель: регистры + явный стек значений (как в VM), базовые блоки по PC.

use crate::байткод::{Инструкция, Функция};

/// Имя экспортируемого символа для функции с индексом `id`.
pub fn имя_символа(id: u32) -> String {
    format!("zip_fn_{}", id)
}

/// Сгенерировать LLVM IR для одной функции (leaf, без Call/Print).
pub fn байткод_в_llvm_ir(функция: &Функция, id: u32) -> Result<String, String> {
    for (i, оп) in функция.код.iter().enumerate() {
        if !поддерживается(оп) {
            return Err(format!(
                "JIT: неподдерживаемая инструкция {:?} @{}",
                оп, i
            ));
        }
    }

    let макс_рег = макс_регистр(функция).max(3);
    let n_regs = (макс_рег as usize) + 1;
    let символ = имя_символа(id);
    let mut out = String::new();

    out.push_str(&format!(
        "; JIT IR для «{}» (id={})\n",
        функция.имя, id
    ));
    out.push_str("target triple = \"x86_64-pc-windows-msvc\"\n\n");
    out.push_str(&format!(
        "define dllexport i64 @{}(i64 %a0, i64 %a1, i64 %a2, i64 %a3) {{\n",
        символ
    ));
    out.push_str("entry:\n");
    out.push_str(&format!(
        "  %regs = alloca [{} x i64], align 8\n",
        n_regs
    ));
    out.push_str("  %stack = alloca [256 x i64], align 8\n");
    out.push_str("  %sp = alloca i64, align 8\n");
    out.push_str("  store i64 0, ptr %sp, align 8\n");

    // Обнулить регистры
    for i in 0..n_regs {
        out.push_str(&format!(
            "  %ri_init{i} = getelementptr [{n_regs} x i64], ptr %regs, i64 0, i64 {i}\n"
        ));
        out.push_str(&format!("  store i64 0, ptr %ri_init{i}, align 8\n"));
    }

    // Аргументы → r0..r3 (по числу параметров)
    let argc = функция.параметры.min(4) as usize;
    let args = ["%a0", "%a1", "%a2", "%a3"];
    for i in 0..argc {
        out.push_str(&format!(
            "  %argptr{i} = getelementptr [{n_regs} x i64], ptr %regs, i64 0, i64 {i}\n"
        ));
        out.push_str(&format!(
            "  store i64 {}, ptr %argptr{i}, align 8\n",
            args[i]
        ));
    }

    if функция.код.is_empty() {
        out.push_str("  ret i64 0\n}\n");
        return Ok(out);
    }

    out.push_str("  br label %bb0\n\n");

    let mut tmp = 0u32;
    for (pc, оп) in функция.код.iter().enumerate() {
        out.push_str(&format!("bb{}:\n", pc));
        эмит_оп(
            &mut out,
            оп,
            pc,
            функция.код.len(),
            n_regs,
            &mut tmp,
        )?;
        out.push('\n');
    }

    out.push_str("}\n");
    Ok(out)
}

fn поддерживается(оп: &Инструкция) -> bool {
    !matches!(
        оп,
        Инструкция::Call(_)
            | Инструкция::Print
            | Инструкция::PrintLn
            | Инструкция::PrintI64
            | Инструкция::ChoirSpawn(_)
            | Инструкция::ChoirJoin
            | Инструкция::PulseWait(_)
            | Инструкция::PerfectLock(_)
            | Инструкция::PushF64(_)
    )
}

fn макс_регистр(ф: &Функция) -> u16 {
    let mut m = 0u16;
    for оп in &ф.код {
        match оп {
            Инструкция::Load(р) | Инструкция::Store(р) => m = m.max(*р),
            Инструкция::Pulse { из, в } => m = m.max(*из).max(*в),
            _ => {}
        }
    }
    m
}

fn эмит_оп(
    out: &mut String,
    оп: &Инструкция,
    pc: usize,
    len: usize,
    n_regs: usize,
    tmp: &mut u32,
) -> Result<(), String> {
    let next = pc + 1;
    match оп {
        Инструкция::Nop => {
            br_next(out, next, len);
        }
        Инструкция::PushI64(n) => {
            let v = t(tmp);
            out.push_str(&format!("  %{v} = add i64 0, {n}\n"));
            push(out, &format!("%{v}"), tmp);
            br_next(out, next, len);
        }
        Инструкция::Load(р) => {
            let p = t(tmp);
            let v = t(tmp);
            out.push_str(&format!(
                "  %{p} = getelementptr [{n_regs} x i64], ptr %regs, i64 0, i64 {р}\n"
            ));
            out.push_str(&format!("  %{v} = load i64, ptr %{p}, align 8\n"));
            push(out, &format!("%{v}"), tmp);
            br_next(out, next, len);
        }
        Инструкция::Store(р) => {
            let v = pop(out, tmp);
            let p = t(tmp);
            out.push_str(&format!(
                "  %{p} = getelementptr [{n_regs} x i64], ptr %regs, i64 0, i64 {р}\n"
            ));
            out.push_str(&format!("  store i64 %{v}, ptr %{p}, align 8\n"));
            br_next(out, next, len);
        }
        Инструкция::Add => binop(out, "add", next, len, tmp),
        Инструкция::Sub => binop(out, "sub", next, len, tmp),
        Инструкция::Mul => binop(out, "mul", next, len, tmp),
        Инструкция::Div => binop(out, "sdiv", next, len, tmp),
        Инструкция::CmpEq => cmp(out, "eq", next, len, tmp),
        Инструкция::CmpNe => cmp(out, "ne", next, len, tmp),
        Инструкция::CmpLt => cmp(out, "slt", next, len, tmp),
        Инструкция::CmpGt => cmp(out, "sgt", next, len, tmp),
        Инструкция::CmpLe => cmp(out, "sle", next, len, tmp),
        Инструкция::CmpGe => cmp(out, "sge", next, len, tmp),
        Инструкция::Pulse { из, в } => {
            let p_из = t(tmp);
            let p_в = t(tmp);
            let v = t(tmp);
            out.push_str(&format!(
                "  %{p_из} = getelementptr [{n_regs} x i64], ptr %regs, i64 0, i64 {из}\n"
            ));
            out.push_str(&format!(
                "  %{p_в} = getelementptr [{n_regs} x i64], ptr %regs, i64 0, i64 {в}\n"
            ));
            out.push_str(&format!("  %{v} = load i64, ptr %{p_из}, align 8\n"));
            out.push_str(&format!("  store i64 %{v}, ptr %{p_в}, align 8\n"));
            out.push_str(&format!("  store i64 0, ptr %{p_из}, align 8\n"));
            br_next(out, next, len);
        }
        Инструкция::Jump(адрес) => {
            out.push_str(&format!("  br label %bb{}\n", *адрес));
        }
        Инструкция::JumpIf(адрес) => {
            let v = pop(out, tmp);
            let c = t(tmp);
            out.push_str(&format!("  %{c} = icmp ne i64 %{v}, 0\n"));
            let else_bb = if next < len {
                format!("bb{}", next)
            } else {
                let end = t(tmp);
                out.push_str(&format!("  br i1 %{c}, label %bb{}, label %retzero_{end}\n", *адрес));
                out.push_str(&format!("retzero_{end}:\n  ret i64 0\n"));
                return Ok(());
            };
            out.push_str(&format!(
                "  br i1 %{c}, label %bb{}, label %{}\n",
                *адрес, else_bb
            ));
        }
        Инструкция::JumpIfNot(адрес) => {
            let v = pop(out, tmp);
            let c = t(tmp);
            out.push_str(&format!("  %{c} = icmp eq i64 %{v}, 0\n"));
            let else_bb = if next < len {
                format!("bb{}", next)
            } else {
                "bb_end".into()
            };
            if next >= len {
                out.push_str(&format!(
                    "  br i1 %{c}, label %bb{}, label %bb_end\n",
                    *адрес
                ));
                out.push_str("bb_end:\n  ret i64 0\n");
            } else {
                out.push_str(&format!(
                    "  br i1 %{c}, label %bb{}, label %{}\n",
                    *адрес, else_bb
                ));
            }
        }
        Инструкция::Ret => {
            let spv = t(tmp);
            let empty = t(tmp);
            let v = t(tmp);
            let slot = t(tmp);
            let spn = t(tmp);
            out.push_str(&format!("  %{spv} = load i64, ptr %sp, align 8\n"));
            out.push_str(&format!("  %{empty} = icmp eq i64 %{spv}, 0\n"));
            out.push_str(&format!(
                "  br i1 %{empty}, label %ret_zero_{pc}, label %ret_pop_{pc}\n"
            ));
            out.push_str(&format!("ret_zero_{pc}:\n  ret i64 0\n"));
            out.push_str(&format!("ret_pop_{pc}:\n"));
            out.push_str(&format!("  %{spn} = sub i64 %{spv}, 1\n"));
            out.push_str(&format!(
                "  %{slot} = getelementptr [256 x i64], ptr %stack, i64 0, i64 %{spn}\n"
            ));
            out.push_str(&format!("  %{v} = load i64, ptr %{slot}, align 8\n"));
            out.push_str(&format!("  store i64 %{spn}, ptr %sp, align 8\n"));
            out.push_str(&format!("  ret i64 %{v}\n"));
        }
        другой => {
            return Err(format!("JIT: внутренняя ошибка, {:?}", другой));
        }
    }
    Ok(())
}

fn binop(out: &mut String, op: &str, next: usize, len: usize, tmp: &mut u32) {
    let b = pop(out, tmp);
    let a = pop(out, tmp);
    let r = t(tmp);
    out.push_str(&format!("  %{r} = {op} i64 %{a}, %{b}\n"));
    push(out, &format!("%{r}"), tmp);
    br_next(out, next, len);
}

fn cmp(out: &mut String, pred: &str, next: usize, len: usize, tmp: &mut u32) {
    let b = pop(out, tmp);
    let a = pop(out, tmp);
    let c = t(tmp);
    let r = t(tmp);
    out.push_str(&format!("  %{c} = icmp {pred} i64 %{a}, %{b}\n"));
    out.push_str(&format!("  %{r} = zext i1 %{c} to i64\n"));
    push(out, &format!("%{r}"), tmp);
    br_next(out, next, len);
}

fn push(out: &mut String, val: &str, tmp: &mut u32) {
    let spv = t(tmp);
    let slot = t(tmp);
    let spn = t(tmp);
    out.push_str(&format!("  %{spv} = load i64, ptr %sp, align 8\n"));
    out.push_str(&format!(
        "  %{slot} = getelementptr [256 x i64], ptr %stack, i64 0, i64 %{spv}\n"
    ));
    out.push_str(&format!("  store i64 {val}, ptr %{slot}, align 8\n"));
    out.push_str(&format!("  %{spn} = add i64 %{spv}, 1\n"));
    out.push_str(&format!("  store i64 %{spn}, ptr %sp, align 8\n"));
}

fn pop(out: &mut String, tmp: &mut u32) -> String {
    let spv = t(tmp);
    let spn = t(tmp);
    let slot = t(tmp);
    let v = t(tmp);
    out.push_str(&format!("  %{spv} = load i64, ptr %sp, align 8\n"));
    out.push_str(&format!("  %{spn} = sub i64 %{spv}, 1\n"));
    out.push_str(&format!(
        "  %{slot} = getelementptr [256 x i64], ptr %stack, i64 0, i64 %{spn}\n"
    ));
    out.push_str(&format!("  %{v} = load i64, ptr %{slot}, align 8\n"));
    out.push_str(&format!("  store i64 %{spn}, ptr %sp, align 8\n"));
    v
}

fn br_next(out: &mut String, next: usize, len: usize) {
    if next < len {
        out.push_str(&format!("  br label %bb{}\n", next));
    } else {
        out.push_str("  ret i64 0\n");
    }
}

fn t(tmp: &mut u32) -> String {
    let n = *tmp;
    *tmp += 1;
    format!("t{}", n)
}

#[cfg(test)]
mod тесты {
    use super::*;
    use crate::байткод::Инструкция;

    #[test]
    fn простой_add_ir() {
        let ф = Функция {
            имя: "сложить".into(),
            параметры: 2,
            локальные: 8,
            размер_стека: 16,
            порог_горячности: 1,
            код: vec![
                Инструкция::Load(0),
                Инструкция::Load(1),
                Инструкция::Add,
                Инструкция::Ret,
            ],
        };
        let ir = байткод_в_llvm_ir(&ф, 7).unwrap();
        assert!(ir.contains("define dllexport i64 @zip_fn_7"));
        assert!(ir.contains("add i64"));
        assert!(ir.contains("ret i64"));
    }
}
