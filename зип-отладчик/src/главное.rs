//! ЗИП-отладчик — DAP-адаптер с пошаговой интерпретацией AST.

use std::collections::{HashMap, HashSet};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use dap::prelude::*;
use dap::requests::SetBreakpointsArguments;
use dap::types::StoppedEventReason;
use dap::responses::{
    ContinueResponse, EvaluateResponse, SetBreakpointsResponse, StackTraceResponse,
    ThreadsResponse, VariablesResponse,
};
use dap::types::{
    Breakpoint, Capabilities, Scope, Source, StackFrame, Thread, Variable,
};
use zipc::{разобрать_исходник, Узел};

// ---------------------------------------------------------------------------
// Значения и выражения
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Знач {
    Целое(i64),
    Строка(String),
    Пусто,
}

impl Знач {
    fn показать(&self) -> String {
        match self {
            Знач::Целое(n) => n.to_string(),
            Знач::Строка(s) => format!("\"{s}\""),
            Знач::Пусто => "пусто".into(),
        }
    }
    fn тип_имени(&self) -> &'static str {
        match self {
            Знач::Целое(_) => "целое",
            Знач::Строка(_) => "строка",
            Знач::Пусто => "пусто",
        }
    }
    fn как_целое(&self) -> Result<i64> {
        match self {
            Знач::Целое(n) => Ok(*n),
            _ => bail!("ожидалось целое, получено {}", self.показать()),
        }
    }
}

#[derive(Debug, Clone)]
enum Выр {
    Целое(i64),
    Строка(String),
    Имя(String),
    Бинар(String, Box<Выр>, Box<Выр>),
    Вызов(String, Vec<Выр>),
}

#[derive(Debug, Clone)]
enum Шаг {
    Присвоить { имя: String, выр: Выр, строка: u32 },
    Выражение { выр: Выр, строка: u32 },
    Возврат { выр: Option<Выр>, строка: u32 },
    ВызовФункции {
        имя: String,
        аргументы: Vec<Выр>,
        строка: u32,
    },
}

#[derive(Debug, Clone)]
struct ФункцияОтл {
    параметры: Vec<String>,
    тело: Vec<Шаг>,
}

#[derive(Debug)]
struct Кадр {
    имя: String,
    шаги: Vec<Шаг>,
    pc: usize,
    локальные: HashMap<String, Знач>,
    строка: u32,
    /// Куда записать результат вложенного вызова (`x = f()`).
    приёмник_возврата: Option<String>,
}

#[derive(Debug)]
struct Сессия {
    файл: String,
    исходник: String,
    функции: HashMap<String, ФункцияОтл>,
    стек: Vec<Кадр>,
    глобальные: HashMap<String, Знач>,
    точки: HashSet<u32>,
    остановлена: bool,
    завершена: bool,
    причина: String,
    вывод: Vec<String>,
    /// После stepIn/next ждём остановки на следующей инструкции
    режим_шага: РежимШага,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum РежимШага {
    Нет,
    Next,
    StepIn,
    Continue,
}

impl Сессия {
    fn из_файла(путь: &str) -> Result<Self> {
        let исходник = std::fs::read_to_string(путь)
            .with_context(|| format!("не удалось прочитать {путь}"))?;
        let дерево = разобрать_исходник(&исходник).map_err(|е| anyhow::anyhow!("{е}"))?;
        let mut функции = HashMap::new();
        собрать_функции(&дерево, &исходник, &mut функции);
        // Точка входа: главная() или тело программы
        let вход = if функции.contains_key("главная") {
            vec![Шаг::ВызовФункции {
                имя: "главная".into(),
                аргументы: vec![],
                строка: найти_строку(&исходник, "главная").unwrap_or(1),
            }]
        } else {
            собрать_тело(&дерево, &исходник)
        };
        let строка = match вход.first() {
            Some(Шаг::Присвоить { строка, .. })
            | Some(Шаг::Выражение { строка, .. })
            | Some(Шаг::Возврат { строка, .. })
            | Some(Шаг::ВызовФункции { строка, .. }) => *строка,
            None => 1,
        };
        Ok(Self {
            файл: путь.into(),
            исходник,
            функции,
            стек: vec![Кадр {
                имя: "главная".into(),
                шаги: вход,
                pc: 0,
                локальные: HashMap::new(),
                строка,
                приёмник_возврата: None,
            }],
            глобальные: HashMap::new(),
            точки: HashSet::new(),
            остановлена: true,
            завершена: false,
            причина: "entry".into(),
            вывод: Vec::new(),
            режим_шага: РежимШага::Нет,
        })
    }

    fn текущая_строка(&self) -> u32 {
        self.стек.last().map(|к| к.строка).unwrap_or(1)
    }

    fn обновить_строку_кадра(&mut self) {
        if let Some(к) = self.стек.last_mut() {
            if let Some(ш) = к.шаги.get(к.pc) {
                к.строка = match ш {
                    Шаг::Присвоить { строка, .. }
                    | Шаг::Выражение { строка, .. }
                    | Шаг::Возврат { строка, .. }
                    | Шаг::ВызовФункции { строка, .. } => *строка,
                };
            }
        }
    }

    fn переменные_текущие(&self) -> Vec<(String, Знач)> {
        let mut out = Vec::new();
        if let Some(к) = self.стек.last() {
            let mut keys: Vec<_> = к.локальные.keys().cloned().collect();
            keys.sort();
            for k in keys {
                out.push((k.clone(), к.локальные[&k].clone()));
            }
        }
        let mut gkeys: Vec<_> = self.глобальные.keys().cloned().collect();
        gkeys.sort();
        for k in gkeys {
            if !out.iter().any(|(n, _)| n == &k) {
                out.push((k.clone(), self.глобальные[&k].clone()));
            }
        }
        out
    }

    fn вычислить(&self, выр: &Выр, локальные: &HashMap<String, Знач>) -> Result<Знач> {
        match выр {
            Выр::Целое(n) => Ok(Знач::Целое(*n)),
            Выр::Строка(s) => Ok(Знач::Строка(s.clone())),
            Выр::Имя(имя) => локальные
                .get(имя)
                .or_else(|| self.глобальные.get(имя))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("неизвестная переменная «{имя}»")),
            Выр::Бинар(оп, а, б) => {
                let av = self.вычислить(а, локальные)?.как_целое()?;
                let bv = self.вычислить(б, локальные)?.как_целое()?;
                let r = match оп.as_str() {
                    "+" => av + bv,
                    "-" => av - bv,
                    "*" => av * bv,
                    "/" => {
                        if bv == 0 {
                            bail!("деление на ноль");
                        }
                        av / bv
                    }
                    _ => bail!("неизвестный оператор {оп}"),
                };
                Ok(Знач::Целое(r))
            }
            Выр::Вызов(имя, арг) => {
                // встроенные
                if имя == "печать_число" || имя == "ПечатьЧисло" {
                    let v = self.вычислить(&арг[0], локальные)?;
                    return Ok(v);
                }
                if имя == "печать_строки" || имя == "ПечатьСтроки" || имя == "печать" {
                    let v = self.вычислить(&арг[0], локальные)?;
                    return Ok(v);
                }
                bail!("вызов «{имя}» нельзя вычислить в evaluate (используйте stepIn)")
            }
        }
    }

    /// Выполнить один шаг. Возвращает true если нужно остановиться.
    fn шаг(&mut self) -> Result<bool> {
        if self.завершена {
            return Ok(true);
        }
        self.обновить_строку_кадра();
        let строка = self.текущая_строка();

        // Остановка на точке до выполнения
        if self.точки.contains(&строка) && self.режим_шага == РежимШага::Continue {
            self.остановлена = true;
            self.причина = "breakpoint".into();
            self.режим_шага = РежимШага::Нет;
            return Ok(true);
        }

        let кадр_idx = self.стек.len() - 1;
        let шаг = {
            let к = &self.стек[кадр_idx];
            if к.pc >= к.шаги.len() {
                // выход из кадра
                let возвр = Знач::Пусто;
                self.стек.pop();
                if self.стек.is_empty() {
                    self.завершена = true;
                    self.остановлена = true;
                    self.причина = "exit".into();
                    return Ok(true);
                }
                // результат вызова пока игнорируем (нет присваивания из вызова в шаге)
                let _ = возвр;
                self.обновить_строку_кадра();
                return self.после_шага();
            }
            к.шаги[к.pc].clone()
        };

        match шаг {
            Шаг::Присвоить { имя, выр, .. } => {
                // x = f(...) — войти в функцию, результат записать при возврате
                if let Выр::Вызов(ф_имя, арги) = &выр {
                    if self.функции.contains_key(ф_имя) {
                        self.стек[кадр_idx].приёмник_возврата = Some(имя);
                        self.стек[кадр_idx].pc += 1;
                        return self.войти_в_функцию(ф_имя, арги, строка);
                    }
                }
                let локал = self.стек[кадр_idx].локальные.clone();
                let зн = match self.вычислить(&выр, &локал) {
                    Ok(v) => v,
                    Err(е) => {
                        self.остановлена = true;
                        self.причина = format!("exception: {е}");
                        return Ok(true);
                    }
                };
                self.стек[кадр_idx].локальные.insert(имя, зн);
                self.стек[кадр_idx].pc += 1;
            }
            Шаг::Выражение { выр, .. } => {
                let локал = self.стек[кадр_idx].локальные.clone();
                match &выр {
                    Выр::Вызов(имя, арги)
                        if имя == "печать_число"
                            || имя == "ПечатьЧисло"
                            || имя == "печать_строки"
                            || имя == "ПечатьСтроки"
                            || имя == "печать" =>
                    {
                        match self.вычислить(&выр, &локал) {
                            Ok(v) => self.вывод.push(v.показать()),
                            Err(е) => {
                                self.остановлена = true;
                                self.причина = format!("exception: {е}");
                                return Ok(true);
                            }
                        }
                        self.стек[кадр_idx].pc += 1;
                    }
                    Выр::Вызов(имя, арги) if self.функции.contains_key(имя) => {
                        // преобразовать в вызов функции
                        self.стек[кадр_idx].pc += 1;
                        return self.войти_в_функцию(имя, арги, строка);
                    }
                    _ => {
                        if let Err(е) = self.вычислить(&выр, &локал) {
                            self.остановлена = true;
                            self.причина = format!("exception: {е}");
                            return Ok(true);
                        }
                        self.стек[кадр_idx].pc += 1;
                    }
                }
            }
            Шаг::Возврат { выр, .. } => {
                let локал = self.стек[кадр_idx].локальные.clone();
                let зн = if let Some(e) = выр {
                    match self.вычислить(&e, &локал) {
                        Ok(v) => v,
                        Err(е) => {
                            self.остановлена = true;
                            self.причина = format!("exception: {е}");
                            return Ok(true);
                        }
                    }
                } else {
                    Знач::Пусто
                };
                self.стек.pop();
                if self.стек.is_empty() {
                    self.глобальные.insert("__результат".into(), зн);
                    self.завершена = true;
                    self.остановлена = true;
                    self.причина = "exit".into();
                    return Ok(true);
                }
                if let Some(приёмник) = self.стек.last_mut().and_then(|к| к.приёмник_возврата.take())
                {
                    self.стек
                        .last_mut()
                        .unwrap()
                        .локальные
                        .insert(приёмник, зн);
                }
            }
            Шаг::ВызовФункции {
                имя, аргументы, ..
            } => {
                self.стек[кадр_idx].pc += 1;
                return self.войти_в_функцию(&имя, &аргументы, строка);
            }
        }

        self.обновить_строку_кадра();
        self.после_шага()
    }

    fn войти_в_функцию(&mut self, имя: &str, аргументы: &[Выр], строка: u32) -> Result<bool> {
        let ф = self
            .функции
            .get(имя)
            .ok_or_else(|| anyhow::anyhow!("нет функции «{имя}»"))?
            .clone();
        let родитель_лок = self
            .стек
            .last()
            .map(|к| к.локальные.clone())
            .unwrap_or_default();
        let mut локальные = HashMap::new();
        for (i, п) in ф.параметры.iter().enumerate() {
            let зн = if let Some(а) = аргументы.get(i) {
                self.вычислить(а, &родитель_лок)?
            } else {
                Знач::Пусто
            };
            локальные.insert(п.clone(), зн);
        }
        let стр = ф.тело.first().map(|ш| match ш {
            Шаг::Присвоить { строка, .. }
            | Шаг::Выражение { строка, .. }
            | Шаг::Возврат { строка, .. }
            | Шаг::ВызовФункции { строка, .. } => *строка,
        }).unwrap_or(строка);
        self.стек.push(Кадр {
            имя: имя.into(),
            шаги: ф.тело,
            pc: 0,
            локальные,
            строка: стр,
            приёмник_возврата: None,
        });
        self.после_шага()
    }

    fn после_шага(&mut self) -> Result<bool> {
        self.обновить_строку_кадра();
        let строка = self.текущая_строка();
        match self.режим_шага {
            РежимШага::Next | РежимШага::StepIn => {
                self.остановлена = true;
                self.причина = "step".into();
                self.режим_шага = РежимШага::Нет;
                Ok(true)
            }
            РежимШага::Continue => {
                if self.точки.contains(&строка) {
                    self.остановлена = true;
                    self.причина = "breakpoint".into();
                    self.режим_шага = РежимШага::Нет;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            РежимШага::Нет => Ok(true),
        }
    }

    fn бежать_до_стопа(&mut self) -> Result<()> {
        self.остановлена = false;
        loop {
            if self.завершена {
                self.остановлена = true;
                break;
            }
            if self.шаг()? {
                break;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Сборка шагов из AST
// ---------------------------------------------------------------------------

fn найти_строку(исходник: &str, игла: &str) -> Option<u32> {
    исходник
        .lines()
        .enumerate()
        .find(|(_, л)| л.contains(игла))
        .map(|(i, _)| (i + 1) as u32)
}

fn узел_в_выр(у: &Узел) -> Result<Выр> {
    match у {
        Узел::Целое(n) => Ok(Выр::Целое(*n)),
        Узел::Строка(s) => Ok(Выр::Строка(s.clone())),
        Узел::Идентификатор(и) => Ok(Выр::Имя(и.clone())),
        Узел::БинарнаяОперация {
            оператор,
            левый,
            правый,
        } => Ok(Выр::Бинар(
            оператор.clone(),
            Box::new(узел_в_выр(левый)?),
            Box::new(узел_в_выр(правый)?),
        )),
        Узел::ВызовФункции { имя, аргументы } => Ok(Выр::Вызов(
            имя.clone(),
            аргументы
                .iter()
                .map(узел_в_выр)
                .collect::<Result<Vec<_>>>()?,
        )),
        Узел::ВызовГлагола {
            глагол,
            аргументы,
            ..
        } => Ok(Выр::Вызов(
            глагол.clone(),
            аргументы
                .iter()
                .map(узел_в_выр)
                .collect::<Result<Vec<_>>>()?,
        )),
        другой => bail!("выражение не поддерживается в отладчике: {другой:?}"),
    }
}

fn собрать_тело(узел: &Узел, исходник: &str) -> Vec<Шаг> {
    let mut out = Vec::new();
    собрать_шаги(узел, исходник, &mut out);
    out
}

fn собрать_шаги(узел: &Узел, исходник: &str, out: &mut Vec<Шаг>) {
    match узел {
        Узел::Программа { объявления } => {
            for д in объявления {
                // определения функций пропускаем на верхнем уровне
                if matches!(д, Узел::ОпределениеФункции { .. }) {
                    continue;
                }
                собрать_шаги(д, исходник, out);
            }
        }
        Узел::Блок(эл) => {
            for е in эл {
                собрать_шаги(е, исходник, out);
            }
        }
        Узел::Присваивание { имя, значение } => {
            if let Ok(выр) = узел_в_выр(значение) {
                let строка = найти_строку(исходник, &format!("{имя}")).unwrap_or(1);
                out.push(Шаг::Присвоить {
                    имя: имя.clone(),
                    выр,
                    строка,
                });
            }
        }
        Узел::Возврат(з) => {
            let выр = з.as_ref().and_then(|u| узел_в_выр(u).ok());
            let строка = найти_строку(исходник, "возврат").unwrap_or(1);
            out.push(Шаг::Возврат { выр, строка });
        }
        Узел::ВызовФункции { имя, аргументы } => {
            let арг: Vec<_> = аргументы.iter().filter_map(|a| узел_в_выр(a).ok()).collect();
            let строка = найти_строку(исходник, имя).unwrap_or(1);
            if имя == "печать_число"
                || имя == "ПечатьЧисло"
                || имя == "печать_строки"
                || имя == "ПечатьСтроки"
                || имя == "печать"
            {
                out.push(Шаг::Выражение {
                    выр: Выр::Вызов(имя.clone(), арг),
                    строка,
                });
            } else {
                out.push(Шаг::ВызовФункции {
                    имя: имя.clone(),
                    аргументы: арг,
                    строка,
                });
            }
        }
        Узел::ВызовГлагола {
            глагол,
            аргументы,
            следующий,
        } => {
            let арг: Vec<_> = аргументы.iter().filter_map(|a| узел_в_выр(a).ok()).collect();
            let строка = найти_строку(исходник, глагол).unwrap_or(1);
            out.push(Шаг::Выражение {
                выр: Выр::Вызов(глагол.clone(), арг),
                строка,
            });
            if let Some(с) = следующий {
                собрать_шаги(с, исходник, out);
            }
        }
        _ => {}
    }
}

fn собрать_функции(узел: &Узел, исходник: &str, out: &mut HashMap<String, ФункцияОтл>) {
    match узел {
        Узел::Программа { объявления } | Узел::Блок(объявления) => {
            for д in объявления {
                собрать_функции(д, исходник, out);
            }
        }
        Узел::ОпределениеФункции {
            имя,
            параметры,
            тело,
            ..
        } => {
            let параметры: Vec<_> = параметры.iter().map(|(n, _)| n.clone()).collect();
            let тело_шаги = собрать_тело(тело, исходник);
            out.insert(
                имя.clone(),
                ФункцияОтл {
                    параметры,
                    тело: тело_шаги,
                },
            );
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// DAP
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    eprintln!("ЗИП-ОТЛАДЧИК v0.1.0");
    eprintln!("DAP stdio — ожидание клиента...");

    let input = BufReader::new(std::io::stdin());
    let output = BufWriter::new(std::io::stdout());
    let mut server = Server::new(input, output);
    let mut сессия: Option<Сессия> = None;

    loop {
        let req = match server.poll_request()? {
            Some(r) => r,
            None => break,
        };

        match &req.command {
            Command::Initialize(_) => {
                let rsp = req.success(ResponseBody::Initialize(Capabilities {
                    supports_configuration_done_request: Some(true),
                    supports_evaluate_for_hovers: Some(true),
                    supports_conditional_breakpoints: Some(false),
                    supports_step_back: Some(false),
                    supports_set_variable: Some(false),
                    ..Default::default()
                }));
                server.respond(rsp)?;
                server.send_event(Event::Initialized)?;
            }
            Command::Launch(args) => {
                let программа = извлечь_программу(args).unwrap_or_else(|| "main.zip".into());
                match Сессия::из_файла(&программа) {
                    Ok(s) => {
                        сессия = Some(s);
                        let rsp = req.success(ResponseBody::Launch);
                        server.respond(rsp)?;
                    }
                    Err(е) => {
                        let rsp = req.error(&format!("launch failed: {е}"));
                        server.respond(rsp)?;
                    }
                }
            }
            Command::SetBreakpoints(args) => {
                let точки = установить_точки(&mut сессия, args);
                let rsp = req.success(ResponseBody::SetBreakpoints(SetBreakpointsResponse {
                    breakpoints: точки,
                }));
                server.respond(rsp)?;
            }
            Command::ConfigurationDone => {
                let rsp = req.success(ResponseBody::ConfigurationDone);
                server.respond(rsp)?;
                if let Some(s) = сессия.as_mut() {
                    // Остановка на входе (stopped entry)
                    s.остановлена = true;
                    s.причина = "entry".into();
                    послать_stopped(&mut server, &s.причина)?;
                }
            }
            Command::Threads => {
                let rsp = req.success(ResponseBody::Threads(ThreadsResponse {
                    threads: vec![Thread {
                        id: 1,
                        name: "главный".into(),
                    }],
                }));
                server.respond(rsp)?;
            }
            Command::StackTrace(_) => {
                let frames = стек_frames(&сессия);
                let total = frames.len() as u64;
                let rsp = req.success(ResponseBody::StackTrace(StackTraceResponse {
                    stack_frames: frames,
                    total_frames: Some(total as i64),
                }));
                server.respond(rsp)?;
            }
            Command::Scopes(_) => {
                let rsp = req.success(ResponseBody::Scopes(dap::responses::ScopesResponse {
                    scopes: vec![Scope {
                        name: "Локальные".into(),
                        variables_reference: 1,
                        expensive: false,
                        ..Default::default()
                    }],
                }));
                server.respond(rsp)?;
            }
            Command::Variables(_) => {
                let vars = переменные_dap(&сессия);
                let rsp = req.success(ResponseBody::Variables(VariablesResponse {
                    variables: vars,
                }));
                server.respond(rsp)?;
            }
            Command::Continue(_) => {
                if let Some(s) = сессия.as_mut() {
                    s.режим_шага = РежимШага::Continue;
                    // сдвинуться с текущей строки точки, чтобы не зациклиться
                    let _ = s.шаг();
                    s.бежать_до_стопа()?;
                }
                let rsp = req.success(ResponseBody::Continue(ContinueResponse {
                    all_threads_continued: Some(true),
                }));
                server.respond(rsp)?;
                if let Some(s) = &сессия {
                    if s.завершена {
                        server.send_event(Event::Terminated(None))?;
                    } else {
                        послать_stopped(&mut server, &s.причина)?;
                    }
                }
            }
            Command::Next(_) => {
                if let Some(s) = сессия.as_mut() {
                    s.режим_шага = РежимШага::Next;
                    s.бежать_до_стопа()?;
                }
                let rsp = req.success(ResponseBody::Next);
                server.respond(rsp)?;
                if let Some(s) = &сессия {
                    if s.завершена {
                        server.send_event(Event::Terminated(None))?;
                    } else {
                        послать_stopped(&mut server, &s.причина)?;
                    }
                }
            }
            Command::StepIn(_) => {
                if let Some(s) = сессия.as_mut() {
                    s.режим_шага = РежимШага::StepIn;
                    s.бежать_до_стопа()?;
                }
                let rsp = req.success(ResponseBody::StepIn);
                server.respond(rsp)?;
                if let Some(s) = &сессия {
                    if s.завершена {
                        server.send_event(Event::Terminated(None))?;
                    } else {
                        послать_stopped(&mut server, &s.причина)?;
                    }
                }
            }
            Command::StepOut(_) => {
                if let Some(s) = сессия.as_mut() {
                    let глубина = s.стек.len().saturating_sub(1);
                    s.режим_шага = РежимШага::Continue;
                    while !s.завершена && s.стек.len() > глубина {
                        if s.шаг()? && s.причина.starts_with("exception") {
                            break;
                        }
                    }
                    s.остановлена = true;
                    if !s.причина.starts_with("exception") {
                        s.причина = "step".into();
                    }
                }
                let rsp = req.success(ResponseBody::StepOut);
                server.respond(rsp)?;
                if let Some(s) = &сессия {
                    if s.завершена {
                        server.send_event(Event::Terminated(None))?;
                    } else {
                        послать_stopped(&mut server, &s.причина)?;
                    }
                }
            }
            Command::Evaluate(args) => {
                let value = оценить(&сессия, &args.expression);
                let rsp = req.success(ResponseBody::Evaluate(EvaluateResponse {
                    result: value,
                    type_field: Some("целое".into()),
                    presentation_hint: None,
                    variables_reference: 0,
                    named_variables: None,
                    indexed_variables: None,
                    memory_reference: None,
                }));
                server.respond(rsp)?;
            }
            Command::Disconnect(_) => {
                let rsp = req.success(ResponseBody::Disconnect);
                server.respond(rsp)?;
                break;
            }
            Command::Terminate(_) => {
                let rsp = req.success(ResponseBody::Terminate);
                server.respond(rsp)?;
                break;
            }
            other => {
                eprintln!("необработанная команда: {other:?}");
                let rsp = req.error("unsupported");
                let _ = server.respond(rsp);
            }
        }
    }

    Ok(())
}

fn извлечь_программу(args: &dap::requests::LaunchRequestArguments) -> Option<String> {
    // VS Code передаёт произвольные поля в additional_data / через serde
    if let Some(data) = &args.additional_data {
        if let Some(v) = data.get("программа").or_else(|| data.get("program")) {
            if let Some(s) = v.as_str() {
                return Some(s.to_string());
            }
        }
    }
    // fallback: переменная окружения
    std::env::var("ZIP_DEBUG_PROGRAM").ok()
}

fn установить_точки(сессия: &mut Option<Сессия>, args: &SetBreakpointsArguments) -> Vec<Breakpoint> {
    let mut out = Vec::new();
    if let Some(s) = сессия.as_mut() {
        s.точки.clear();
        if let Some(bps) = &args.breakpoints {
            for bp in bps {
                let line = bp.line as u32;
                s.точки.insert(line);
                out.push(Breakpoint {
                    id: Some(line as i64),
                    verified: true,
                    message: None,
                    source: Some(args.source.clone()),
                    line: Some(bp.line),
                    column: None,
                    end_line: None,
                    end_column: None,
                    instruction_reference: None,
                    offset: None,
                });
            }
        }
    }
    out
}

fn стек_frames(сессия: &Option<Сессия>) -> Vec<StackFrame> {
    let Some(s) = сессия else {
        return vec![];
    };
    s.стек
        .iter()
        .rev()
        .enumerate()
        .map(|(i, к)| StackFrame {
            id: i as i64,
            name: к.имя.clone(),
            source: Some(Source {
                name: Some(
                    PathBuf::from(&s.файл)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| s.файл.clone()),
                ),
                path: Some(s.файл.clone()),
                ..Default::default()
            }),
            line: к.строка as i64,
            column: 1,
            end_line: None,
            end_column: None,
            can_restart: None,
            instruction_pointer_reference: None,
            module_id: None,
            presentation_hint: None,
        })
        .collect()
}

fn переменные_dap(сессия: &Option<Сессия>) -> Vec<Variable> {
    let Some(s) = сессия else {
        return vec![];
    };
    s.переменные_текущие()
        .into_iter()
        .map(|(имя, зн)| Variable {
            name: имя,
            value: зн.показать(),
            type_field: Some(зн.тип_имени().into()),
            presentation_hint: None,
            evaluate_name: None,
            variables_reference: 0,
            named_variables: None,
            indexed_variables: None,
            memory_reference: None,
        })
        .collect()
}

fn оценить(сессия: &Option<Сессия>, выражение: &str) -> String {
    let Some(s) = сессия else {
        return "?".into();
    };
    let выр = выражение.trim();
    if let Some((_, зн)) = s
        .переменные_текущие()
        .into_iter()
        .find(|(n, _)| n == выр)
    {
        return зн.показать();
    }
    // простое целое
    if let Ok(n) = выр.parse::<i64>() {
        return n.to_string();
    }
    format!("<{выр}>")
}

fn послать_stopped(server: &mut Server<impl std::io::Read, impl std::io::Write>, reason: &str) -> Result<()> {
    use dap::events::StoppedEventBody;
    let reason_enum = if reason.starts_with("exception") {
        StoppedEventReason::Exception
    } else if reason == "breakpoint" {
        StoppedEventReason::Breakpoint
    } else if reason == "entry" {
        StoppedEventReason::Entry
    } else {
        StoppedEventReason::Step
    };
    server.send_event(Event::Stopped(StoppedEventBody {
        reason: reason_enum,
        description: Some(reason.into()),
        thread_id: Some(1),
        preserve_focus_hint: None,
        text: None,
        all_threads_stopped: Some(true),
        hit_breakpoint_ids: None,
    }))?;
    Ok(())
}
