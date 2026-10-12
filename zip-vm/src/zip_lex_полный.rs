//! Полный лексер ZIP — алгоритм на стороне VM (без C-bootstrap).
//! Токены совместимы с `bootstrap/zipb.h` (порядок enum TokenType).

use crate::встроенные_v3::Встроенные;
use crate::исполнитель_v3::ЗначениеV3;
use std::sync::Mutex;

// === Совместимость с bootstrap/zipb.h ===
pub const ТК_КОНЕЦ: i64 = 0;
pub const ТК_ИМПУЛЬС: i64 = 1;
pub const ТК_ХОР: i64 = 2;
pub const ТК_СЕТЬ: i64 = 3;
pub const ТК_ФИКСАЦИЯ: i64 = 4;
pub const ТК_ПОТОК: i64 = 5;
pub const ТК_РАВНО: i64 = 6;
pub const ТК_ДВОЕТОЧИЕ: i64 = 7;
pub const ТК_ЛСКОБКА: i64 = 8;
pub const ТК_ПСКОБКА: i64 = 9;
pub const ТК_ЛФИГУРНАЯ: i64 = 10;
pub const ТК_ПФИГУРНАЯ: i64 = 11;
pub const ТК_ЛКВАД: i64 = 12;
pub const ТК_ПКВАД: i64 = 13;
pub const ТК_ЗАПЯТАЯ: i64 = 14;
pub const ТК_СТРЕЛКА: i64 = 15;
pub const ТК_ДИАПАЗОН: i64 = 16;
pub const ТК_ПЛЮС: i64 = 17;
pub const ТК_МИНУС: i64 = 18;
pub const ТК_ЗВЕЗДА: i64 = 19;
pub const ТК_СЛЕШ: i64 = 20;
pub const ТК_ПРОЦЕНТ: i64 = 21;
pub const ТК_МЕНЬШЕ: i64 = 22;
pub const ТК_МЕНЬШЕ_РАВНО: i64 = 23;
pub const ТК_БОЛЬШЕ: i64 = 24;
pub const ТК_БОЛЬШЕ_РАВНО: i64 = 25;
pub const ТК_РАВНО_РАВНО: i64 = 26;
pub const ТК_НЕ_РАВНО: i64 = 27;
pub const ТК_ЦЕЛОЕ: i64 = 28;
pub const ТК_ВЕЩЕСТВЕННОЕ: i64 = 29;
pub const ТК_СТРОКА: i64 = 30;
pub const ТК_СЛОВО: i64 = 31;
pub const ТК_КОРЕНЬ: i64 = 32;
pub const ТК_ШАБЛОН: i64 = 33;
pub const ТК_ЕСЛИ: i64 = 34;
pub const ТК_ИНАЧЕ: i64 = 35;
pub const ТК_ПОКА: i64 = 36;
pub const ТК_ДЛЯ: i64 = 37;
pub const ТК_В: i64 = 38;
pub const ТК_ВОЗВРАТ: i64 = 39;
pub const ТК_ПРЕРВАТЬ: i64 = 40;
pub const ТК_ПРОДОЛЖИТЬ: i64 = 41;
pub const ТК_ФУНКЦИЯ: i64 = 42;
pub const ТК_ТИП: i64 = 43;
pub const ТК_ГЛАГОЛ: i64 = 44;
pub const ТК_АГЕНТ: i64 = 45;
pub const ТК_ПОТОК_ДАННЫХ: i64 = 46;
pub const ТК_ИСТИНА: i64 = 47;
pub const ТК_ЛОЖЬ: i64 = 48;
pub const ТК_И: i64 = 49;
pub const ТК_ИЛИ: i64 = 50;
pub const ТК_НЕ: i64 = 51;
pub const ТК_НИЧТО: i64 = 52;
pub const ТК_НЕЧТО: i64 = 53;
pub const ТК_УСПЕХ: i64 = 54;
pub const ТК_ОШИБКА: i64 = 55;
pub const ТК_СОБАКА: i64 = 56;
pub const ТК_РЕШЁТКА: i64 = 57;

#[derive(Clone, Debug)]
pub struct Токен {
    pub вид: i64,
    pub целое: i64,
    pub строка: String,
    pub линия: i32,
    pub столбец: i32,
}

struct Лексер<'a> {
    data: &'a [u8],
    pos: usize,
    line: i32,
    col: i32,
}

fn is_cyr(data: &[u8], pos: usize) -> bool {
    if pos + 1 >= data.len() {
        return false;
    }
    let b0 = data[pos];
    let b1 = data[pos + 1];
    (b0 == 0xD0 || b0 == 0xD1) && (0x80..=0xBF).contains(&b1)
}

impl<'a> Лексер<'a> {
    fn cur(&self) -> u8 {
        if self.pos < self.data.len() {
            self.data[self.pos]
        } else {
            0
        }
    }

    fn peek(&self, n: usize) -> u8 {
        let i = self.pos + n;
        if i < self.data.len() {
            self.data[i]
        } else {
            0
        }
    }

    fn adv(&mut self) {
        if self.pos >= self.data.len() {
            return;
        }
        if self.data[self.pos] == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.pos += 1;
    }

    fn skip(&mut self) {
        loop {
            let c = self.cur();
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                self.adv();
                continue;
            }
            if c == b'/' && self.peek(1) == b'/' {
                while self.pos < self.data.len() && self.cur() != b'\n' {
                    self.adv();
                }
                continue;
            }
            if c == b'/' && self.peek(1) == b'*' {
                self.adv();
                self.adv();
                while self.pos + 1 < self.data.len() {
                    if self.cur() == b'*' && self.peek(1) == b'/' {
                        self.adv();
                        self.adv();
                        break;
                    }
                    self.adv();
                }
                continue;
            }
            break;
        }
    }

    fn tok_num(&mut self) -> Токен {
        let start = self.pos;
        let line = self.line;
        let col = self.col;
        let mut dot = false;
        while self.pos < self.data.len() {
            let c = self.cur();
            if c.is_ascii_digit() {
                self.adv();
            } else if c == b'.' && !dot && self.peek(1).is_ascii_digit() {
                dot = true;
                self.adv();
            } else {
                break;
            }
        }
        let s = std::str::from_utf8(&self.data[start..self.pos]).unwrap_or("");
        if dot {
            Токен {
                вид: ТК_ВЕЩЕСТВЕННОЕ,
                целое: 0,
                строка: s.to_string(),
                линия: line,
                столбец: col,
            }
        } else {
            Токен {
                вид: ТК_ЦЕЛОЕ,
                целое: s.parse().unwrap_or(0),
                строка: String::new(),
                линия: line,
                столбец: col,
            }
        }
    }

    fn tok_str(&mut self) -> Токен {
        let line = self.line;
        let col = self.col;
        self.adv(); // "
        let mut out = String::new();
        while self.pos < self.data.len() && self.cur() != b'"' {
            if self.cur() == b'\\' && self.pos + 1 < self.data.len() {
                self.adv();
                let e = self.cur();
                out.push(match e {
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    b'\\' => '\\',
                    b'"' => '"',
                    _ => e as char,
                });
                self.adv();
            } else {
                out.push(self.cur() as char);
                self.adv();
            }
        }
        if self.cur() == b'"' {
            self.adv();
        }
        Токен {
            вид: ТК_СТРОКА,
            целое: 0,
            строка: out,
            линия: line,
            столбец: col,
        }
    }

    fn keyword(w: &str) -> i64 {
        match w {
            "если" => ТК_ЕСЛИ,
            "иначе" => ТК_ИНАЧЕ,
            "пока" => ТК_ПОКА,
            "для" => ТК_ДЛЯ,
            "в" => ТК_В,
            "возврат" => ТК_ВОЗВРАТ,
            "прервать" => ТК_ПРЕРВАТЬ,
            "продолжить" => ТК_ПРОДОЛЖИТЬ,
            "функция" => ТК_ФУНКЦИЯ,
            "тип" => ТК_ТИП,
            "глагол" => ТК_ГЛАГОЛ,
            "агент" => ТК_АГЕНТ,
            "поток" => ТК_ПОТОК_ДАННЫХ,
            "истина" => ТК_ИСТИНА,
            "ложь" => ТК_ЛОЖЬ,
            "и" => ТК_И,
            "или" => ТК_ИЛИ,
            "не" => ТК_НЕ,
            "ничто" | "Ничто" => ТК_НИЧТО,
            "нечто" | "Нечто" => ТК_НЕЧТО,
            "успех" | "Успех" => ТК_УСПЕХ,
            "ошибка" | "Ошибка" => ТК_ОШИБКА,
            _ => ТК_СЛОВО,
        }
    }

    fn tok_word(&mut self) -> Токен {
        let start = self.pos;
        let line = self.line;
        let col = self.col;
        let mut cyr_chars = 0;
        while self.pos < self.data.len() {
            if is_cyr(self.data, self.pos) {
                self.adv();
                self.adv();
                cyr_chars += 1;
            } else {
                let c = self.cur();
                if c.is_ascii_alphanumeric() || c == b'_' {
                    self.adv();
                } else {
                    break;
                }
            }
        }
        let n = self.pos - start;
        let w = std::str::from_utf8(&self.data[start..self.pos])
            .unwrap_or("")
            .to_string();
        let mut вид = Self::keyword(&w);
        // 3 кириллические буквы (6 байт UTF-8) → корень
        if вид == ТК_СЛОВО && cyr_chars == 3 && n == 6 {
            вид = ТК_КОРЕНЬ;
        }
        // 5 кириллических букв (10 байт) → шаблон
        if вид == ТК_СЛОВО && cyr_chars == 5 && n == 10 {
            вид = ТК_ШАБЛОН;
        }
        Токен {
            вид,
            целое: 0,
            строка: w,
            линия: line,
            столбец: col,
        }
    }

    fn next_token(&mut self) -> Option<Токен> {
        self.skip();
        if self.pos >= self.data.len() {
            return None;
        }
        let line = self.line;
        let col = self.col;
        let c = self.cur();

        let simple = |вид: i64, lx: &mut Лексер| {
            lx.adv();
            Some(Токен {
                вид,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            })
        };

        if c == b'>' && self.peek(1) == b'>' {
            self.adv();
            self.adv();
            if self.cur() == b'>' {
                self.adv();
                return Some(Токен {
                    вид: ТК_СЕТЬ,
                    целое: 0,
                    строка: String::new(),
                    линия: line,
                    столбец: col,
                });
            }
            return Some(Токен {
                вид: ТК_ИМПУЛЬС,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            });
        }
        if c == b'|' && self.peek(1) == b'|' {
            self.adv();
            self.adv();
            return Some(Токен {
                вид: ТК_ХОР,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            });
        }
        if c == b'=' && self.peek(1) == b'=' {
            self.adv();
            self.adv();
            return Some(Токен {
                вид: ТК_РАВНО_РАВНО,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            });
        }
        if c == b'!' && self.peek(1) == b'=' {
            self.adv();
            self.adv();
            return Some(Токен {
                вид: ТК_НЕ_РАВНО,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            });
        }
        if c == b'!' {
            return simple(ТК_ФИКСАЦИЯ, self);
        }
        if c == b'<' && self.peek(1) == b'=' {
            self.adv();
            self.adv();
            return Some(Токен {
                вид: ТК_МЕНЬШЕ_РАВНО,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            });
        }
        if c == b'>' && self.peek(1) == b'=' {
            self.adv();
            self.adv();
            return Some(Токен {
                вид: ТК_БОЛЬШЕ_РАВНО,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            });
        }
        if c == b'-' && self.peek(1) == b'>' {
            self.adv();
            self.adv();
            return Some(Токен {
                вид: ТК_СТРЕЛКА,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            });
        }
        if c == b'.' && self.peek(1) == b'.' {
            self.adv();
            self.adv();
            return Some(Токен {
                вид: ТК_ДИАПАЗОН,
                целое: 0,
                строка: String::new(),
                линия: line,
                столбец: col,
            });
        }
        if c == b'~' {
            return simple(ТК_ПОТОК, self);
        }
        if c == b'=' {
            return simple(ТК_РАВНО, self);
        }
        if c == b':' {
            return simple(ТК_ДВОЕТОЧИЕ, self);
        }
        if c == b'(' {
            return simple(ТК_ЛСКОБКА, self);
        }
        if c == b')' {
            return simple(ТК_ПСКОБКА, self);
        }
        if c == b'{' {
            return simple(ТК_ЛФИГУРНАЯ, self);
        }
        if c == b'}' {
            return simple(ТК_ПФИГУРНАЯ, self);
        }
        if c == b'[' {
            return simple(ТК_ЛКВАД, self);
        }
        if c == b']' {
            return simple(ТК_ПКВАД, self);
        }
        if c == b',' {
            return simple(ТК_ЗАПЯТАЯ, self);
        }
        if c == b'+' {
            return simple(ТК_ПЛЮС, self);
        }
        if c == b'-' {
            return simple(ТК_МИНУС, self);
        }
        if c == b'*' {
            return simple(ТК_ЗВЕЗДА, self);
        }
        if c == b'/' {
            return simple(ТК_СЛЕШ, self);
        }
        if c == b'%' {
            return simple(ТК_ПРОЦЕНТ, self);
        }
        if c == b'<' {
            return simple(ТК_МЕНЬШЕ, self);
        }
        if c == b'>' {
            return simple(ТК_БОЛЬШЕ, self);
        }
        if c == b'@' {
            return simple(ТК_СОБАКА, self);
        }
        if c == b'#' {
            return simple(ТК_РЕШЁТКА, self);
        }
        if c == b'"' {
            return Some(self.tok_str());
        }
        if c.is_ascii_digit() {
            return Some(self.tok_num());
        }
        if is_cyr(self.data, self.pos)
            || c.is_ascii_alphabetic()
            || c == b'_'
        {
            return Some(self.tok_word());
        }
        // неизвестный — пропускаем (как мягкий режим для тестов)
        self.adv();
        self.next_token()
    }
}

/// Разобрать исходник → список токенов (+ EOF).
pub fn разобрать(src: &str) -> Vec<Токен> {
    let bytes = src.as_bytes();
    let mut lx = Лексер {
        data: bytes,
        pos: 0,
        line: 1,
        col: 1,
    };
    let mut out = Vec::new();
    while let Some(t) = lx.next_token() {
        out.push(t);
    }
    out.push(Токен {
        вид: ТК_КОНЕЦ,
        целое: 0,
        строка: String::new(),
        линия: lx.line,
        столбец: lx.col,
    });
    out
}

static TOKENS: Mutex<Vec<Токен>> = Mutex::new(Vec::new());

fn как_строка(з: &ЗначениеV3) -> String {
    match з {
        ЗначениеV3::Строка(s) => s.clone(),
        другой => другой.to_string(),
    }
}

pub fn зарегистрировать(ф: &mut Встроенные) {
    ф.регистрация_внешняя("zip_lex_полный", |арг| {
        let s = арг.first().map(как_строка).unwrap_or_default();
        let toks = разобрать(&s);
        let n = toks.len() as i64;
        *TOKENS.lock().unwrap() = toks;
        Ok(ЗначениеV3::Целое(n))
    });
    ф.регистрация_внешняя("zip_lex_вид", |арг| {
        let i = арг.first().map(|z| z.как_целое()).unwrap_or(0);
        let toks = TOKENS.lock().unwrap();
        if i < 0 || i as usize >= toks.len() {
            return Ok(ЗначениеV3::Целое(-1));
        }
        Ok(ЗначениеV3::Целое(toks[i as usize].вид))
    });
    ф.регистрация_внешняя("zip_lex_целое", |арг| {
        let i = арг.first().map(|z| z.как_целое()).unwrap_or(0);
        let toks = TOKENS.lock().unwrap();
        if i < 0 || i as usize >= toks.len() {
            return Ok(ЗначениеV3::Целое(0));
        }
        Ok(ЗначениеV3::Целое(toks[i as usize].целое))
    });
    ф.регистрация_внешняя("zip_lex_текст", |арг| {
        let i = арг.first().map(|z| z.как_целое()).unwrap_or(0);
        let toks = TOKENS.lock().unwrap();
        if i < 0 || i as usize >= toks.len() {
            return Ok(ЗначениеV3::Строка(String::new()));
        }
        Ok(ЗначениеV3::Строка(toks[i as usize].строка.clone()))
    });
    ф.регистрация_внешняя("zip_byte_at", |арг| {
        let s = арг.first().map(как_строка).unwrap_or_default();
        let i = арг.get(1).map(|z| z.как_целое()).unwrap_or(0);
        let b = s.as_bytes().get(i as usize).copied().unwrap_or(0);
        Ok(ЗначениеV3::Целое(b as i64))
    });

    for (alias, target) in [
        ("лексер_разобрать_полный", "zip_lex_полный"),
        ("лексер_вид", "zip_lex_вид"),
        ("лексер_целое", "zip_lex_целое"),
        ("лексер_текст", "zip_lex_текст"),
    ] {
        ф.регистрация_алиас(alias, target);
    }
}

#[cfg(test)]
mod тесты {
    use super::*;

    #[test]
    fn пустой() {
        let t = разобрать("");
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].вид, ТК_КОНЕЦ);
    }

    #[test]
    fn число() {
        let t = разобрать("42");
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].вид, ТК_ЦЕЛОЕ);
        assert_eq!(t[0].целое, 42);
    }

    #[test]
    fn функция_kw() {
        let t = разобрать("функция");
        assert_eq!(t[0].вид, ТК_ФУНКЦИЯ);
    }

    #[test]
    fn корень() {
        let t = разобрать("СПР");
        assert_eq!(t[0].вид, ТК_КОРЕНЬ);
    }

    #[test]
    fn шаблон() {
        let t = разобрать("САПАР");
        assert_eq!(t[0].вид, ТК_ШАБЛОН);
    }

    #[test]
    fn операторы() {
        let t = разобрать(">> >>> || ! ~");
        assert_eq!(t.len(), 6);
        assert_eq!(t[0].вид, ТК_ИМПУЛЬС);
        assert_eq!(t[1].вид, ТК_СЕТЬ);
        assert_eq!(t[2].вид, ТК_ХОР);
        assert_eq!(t[3].вид, ТК_ФИКСАЦИЯ);
        assert_eq!(t[4].вид, ТК_ПОТОК);
    }
}
