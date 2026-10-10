//! Каркас inkwell-бэкенда (feature `inkwell-llvm`).
//!
//! На системах без полного LLVM SDK используйте текстовый эмиттер
//! [`crate::llvm_ir`] и флаг `--llvm` (clang -O3).

#![cfg(feature = "inkwell-llvm")]

use crate::промежуток::ФункцияПП;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;

pub struct LlvmGenerator<'ctx> {
    контекст: &'ctx Context,
    модуль: Module<'ctx>,
    строитель: Builder<'ctx>,
}

impl<'ctx> LlvmGenerator<'ctx> {
    pub fn новый(контекст: &'ctx Context, имя: &str) -> Self {
        Self {
            контекст,
            модуль: контекст.create_module(имя),
            строитель: контекст.create_builder(),
        }
    }

    pub fn сгенерировать(&mut self, функции: &[ФункцияПП]) -> Result<(), String> {
        // Полноценная генерация тел — через llvm_ir + clang на Windows runtime;
        // здесь — каркас объявлений и оптимизаций inkwell.
        let i64t = self.контекст.i64_type();
        for ф in функции {
            let имя = if ф.имя == "главная" { "main" } else { &ф.имя };
            let ft = i64t.fn_type(&[], false);
            self.модуль.add_function(имя, ft, None);
        }

        let passes = "default<O3>";
        let opts = PassBuilderOptions::create();
        self.модуль
            .run_passes(passes, &self.машина()?, opts)
            .map_err(|е| е.to_string())?;

        let _ = &self.строитель;
        let _ = функции;
        Ok(())
    }

    fn машина(&self) -> Result<TargetMachine, String> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|е| е.to_string())?;
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|е| е.to_string())?;
        target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Aggressive,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| "TargetMachine".into())
    }

    pub fn записать_объект(&self, путь: &str) -> Result<(), String> {
        let tm = self.машина()?;
        tm.write_to_file(&self.модуль, FileType::Object, путь.as_ref())
            .map_err(|е| е.to_string())
    }
}
