//! ЗИП-LSP — Language Server Protocol для языка ZIP.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use zipc::{разобрать_исходник, Лексер, Токен, ТокенСПозицией, Узел};

#[derive(Debug)]
struct Диагностика {
    сообщение: String,
    диапазон: Range,
    серьёзность: DiagnosticSeverity,
}

#[derive(Debug)]
struct Документ {
    текст: String,
    версия: i32,
    токены: Vec<ТокенСПозицией>,
    дерево: Option<Узел>,
    ошибки: Vec<Диагностика>,
}

struct ЗипLsp {
    клиент: Client,
    документы: Arc<RwLock<HashMap<Url, Документ>>>,
}

fn поз(строка: usize, столбец: usize) -> Position {
    Position {
        line: строка.saturating_sub(1) as u32,
        character: столбец.saturating_sub(1) as u32,
    }
}

fn диапазон_точки(строка: usize, столбец: usize, длина: u32) -> Range {
    let start = поз(строка, столбец);
    Range {
        start,
        end: Position {
            line: start.line,
            character: start.character.saturating_add(длина),
        },
    }
}

fn длина_токена(т: &Токен) -> u32 {
    match т {
        Токен::Импульс => 2,
        Токен::СетьИмпульс => 3,
        Токен::Хор => 2,
        Токен::Фиксация | Токен::Поток | Токен::Равно | Токен::Двоеточие | Токен::Дефис
        | Токен::СкобкаЛевая | Токен::СкобкаПравая | Токен::ФигурнаяЛевая
        | Токен::ФигурнаяПравая | Токен::Запятая | Токен::Плюс | Токен::Звезда
        | Токен::Слеш => 1,
        Токен::РавноРавно | Токен::НеРавно | Токен::БольшеРавно | Токен::МеньшеРавно
        | Токен::Диапазон => 2,
        Токен::Корень(s) | Токен::Шаблон(s) | Токен::Слово(s) | Токен::Строка(s)
        | Токен::Директива(s) => s.chars().count() as u32,
        Токен::Целое(n) => n.to_string().len() as u32,
        Токен::Вещественное(x) => format!("{x}").len() as u32,
        Токен::Если => 4,
        Токен::Иначе => 5,
        Токен::Пока => 4,
        Токен::Для => 3,
        Токен::В => 1,
        Токен::Возврат => 7,
        Токен::Прервать => 8,
        Токен::Продолжить => 10,
        Токен::Функция => 7,
        Токен::Тип => 3,
        Токен::Модуль => 6,
        Токен::Импорт => 6,
        Токен::Печать => 6,
        Токен::ПечатьСтроки => 13,
        Токен::ПечатьЧисло => 12,
        Токен::Истина => 6,
        Токен::Ложь => 4,
        Токен::И => 1,
        Токен::Или => 3,
        Токен::Не => 2,
        Токен::Больше | Токен::Меньше => 1,
        Токен::КонецФайла => 0,
        _ => 1,
    }
}

fn тип_семантики(т: &Токен) -> Option<u32> {
    // legend order: KEYWORD=0 FUNCTION=1 VARIABLE=2 STRING=3 NUMBER=4 OPERATOR=5 COMMENT=6
    match т {
        Токен::Если
        | Токен::Иначе
        | Токен::Пока
        | Токен::Для
        | Токен::В
        | Токен::Возврат
        | Токен::Прервать
        | Токен::Продолжить
        | Токен::Функция
        | Токен::Тип
        | Токен::Модуль
        | Токен::Импорт
        | Токен::Истина
        | Токен::Ложь
        | Токен::И
        | Токен::Или
        | Токен::Не => Some(0),
        Токен::Шаблон(_)
        | Токен::Печать
        | Токен::ПечатьСтроки
        | Токен::ПечатьЧисло => Some(1),
        Токен::Корень(_) | Токен::Слово(_) => Some(2),
        Токен::Строка(_) => Some(3),
        Токен::Целое(_) | Токен::Вещественное(_) => Some(4),
        Токен::Импульс
        | Токен::СетьИмпульс
        | Токен::Хор
        | Токен::Фиксация
        | Токен::Поток
        | Токен::Равно
        | Токен::РавноРавно
        | Токен::НеРавно
        | Токен::Плюс
        | Токен::Дефис
        | Токен::Звезда
        | Токен::Слеш
        | Токен::Больше
        | Токен::Меньше
        | Токен::БольшеРавно
        | Токен::МеньшеРавно => Some(5),
        _ => None,
    }
}

fn hover_для(имя: &str) -> Option<String> {
    let док: &[(&str, &str)] = &[
        ("если", "**если** — условный оператор\n\n```zip\nесли условие {\n  …\n} иначе {\n  …\n}\n```"),
        ("пока", "**пока** — цикл с предусловием"),
        ("для", "**для** — цикл по коллекции\n\n```zip\nдля x в коллекция { … }\n```"),
        ("функция", "**функция** — объявление функции"),
        ("возврат", "**возврат** — выход из функции с значением"),
        ("печать_строки", "**печать_строки**(s) — вывод строки с переводом строки"),
        ("печать_число", "**печать_число**(n) — вывод целого числа"),
        ("печать", "**печать**(s) — вывод без перевода строки"),
        ("САПАР", "**САПАР** — глагол счёта / арифметики потока\n\n```zip\nСАПАР(1, 2) >> ЧАТАВ!\n```"),
        ("ЧАТАВ", "**ЧАТАВ** — фиксация результата потока (`!`)"),
        ("ЖАВАТ", "**ЖАВАТ** — асинхронный поток (`~`)"),
        ("истина", "**истина** — логическая константа"),
        ("ложь", "**ложь** — логическая константа"),
    ];
    док.iter()
        .find(|(k, _)| *k == имя)
        .map(|(_, v)| (*v).to_string())
}

fn слово_в_позиции(текст: &str, позиция: Position) -> Option<(String, Range)> {
    let линия = текст.lines().nth(позиция.line as usize)?;
    let chars: Vec<char> = линия.chars().collect();
    let col = позиция.character as usize;
    if chars.is_empty() {
        return None;
    }
    let mut i = col.min(chars.len().saturating_sub(1));
    let is_id = |c: char| c.is_alphanumeric() || c == '_' || ('А'..='я').contains(&c) || c == 'ё' || c == 'Ё';
    if !is_id(chars[i]) && i > 0 {
        i -= 1;
    }
    if !is_id(chars[i]) {
        return None;
    }
    let mut l = i;
    while l > 0 && is_id(chars[l - 1]) {
        l -= 1;
    }
    let mut r = i + 1;
    while r < chars.len() && is_id(chars[r]) {
        r += 1;
    }
    let слово: String = chars[l..r].iter().collect();
    Some((
        слово,
        Range {
            start: Position {
                line: позиция.line,
                character: l as u32,
            },
            end: Position {
                line: позиция.line,
                character: r as u32,
            },
        },
    ))
}

fn собрать_символы(узел: &Узел, out: &mut Vec<DocumentSymbol>) {
    match узел {
        Узел::Программа { объявления } => {
            for д in объявления {
                собрать_символы(д, out);
            }
        }
        Узел::ОпределениеФункции { имя, тело, .. } => {
            let mut дети = Vec::new();
            собрать_символы(тело, &mut дети);
            out.push(DocumentSymbol {
                name: имя.clone(),
                detail: Some("функция".into()),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                range: Range::default(),
                selection_range: Range::default(),
                children: if дети.is_empty() { None } else { Some(дети) },
            });
        }
        Узел::ОбъявлениеКорня { корень, .. } => {
            out.push(DocumentSymbol {
                name: корень.clone(),
                detail: Some("корень".into()),
                kind: SymbolKind::CONSTANT,
                tags: None,
                deprecated: None,
                range: Range::default(),
                selection_range: Range::default(),
                children: None,
            });
        }
        Узел::Присваивание { имя, .. } => {
            out.push(DocumentSymbol {
                name: имя.clone(),
                detail: Some("переменная".into()),
                kind: SymbolKind::VARIABLE,
                tags: None,
                deprecated: None,
                range: Range::default(),
                selection_range: Range::default(),
                children: None,
            });
        }
        Узел::Блок(эл) | Узел::Хор { ветки: эл } => {
            for е in эл {
                собрать_символы(е, out);
            }
        }
        Узел::Если {
            тогда, иначе, ..
        } => {
            собрать_символы(тогда, out);
            if let Some(и) = иначе {
                собрать_символы(и, out);
            }
        }
        Узел::Пока { тело, .. } | Узел::Для { тело, .. } => собрать_символы(тело, out),
        _ => {}
    }
}

fn найти_определение(узел: &Узел, имя: &str) -> bool {
    match узел {
        Узел::ОпределениеФункции { имя: n, .. } | Узел::Присваивание { имя: n, .. } => n == имя,
        Узел::ОбъявлениеКорня { корень, .. } => корень == имя,
        Узел::Программа { объявления } => объявления.iter().any(|у| найти_определение(у, имя)),
        Узел::Блок(эл) => эл.iter().any(|у| найти_определение(у, имя)),
        _ => false,
    }
}

fn разобрать_документ(текст: String, версия: i32) -> Документ {
    let mut ошибки = Vec::new();

    let токены = match Лексер::новый(&текст).разобрать() {
        Ok(т) => т,
        Err(е) => {
            ошибки.push(Диагностика {
                сообщение: format!("Лексическая ошибка: {}", е.сообщение),
                диапазон: диапазон_точки(е.строка, е.столбец, 1),
                серьёзность: DiagnosticSeverity::ERROR,
            });
            Vec::new()
        }
    };

    let дерево = match разобрать_исходник(&текст) {
        Ok(д) => Some(д),
        Err(е) => {
            ошибки.push(Диагностика {
                сообщение: format!("Ошибка парсинга: {}", е.сообщение),
                диапазон: диапазон_точки(е.строка.max(1), е.столбец.max(1), 1),
                серьёзность: DiagnosticSeverity::ERROR,
            });
            None
        }
    };

    Документ {
        текст,
        версия,
        токены,
        дерево,
        ошибки,
    }
}

async fn опубликовать_диагностику(клиент: &Client, uri: Url, док: &Документ) {
    let диагностики: Vec<Diagnostic> = док
        .ошибки
        .iter()
        .map(|д| Diagnostic {
            range: д.диапазон,
            severity: Some(д.серьёзность),
            source: Some("зип-lsp".into()),
            message: д.сообщение.clone(),
            ..Default::default()
        })
        .collect();
    клиент
        .publish_diagnostics(uri, диагностики, Some(док.версия))
        .await;
}

#[tower_lsp::async_trait]
impl LanguageServer for ЗипLsp {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "ЗИП-LSP".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".into(), ":".into(), "(".into()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::OPERATOR,
                                    SemanticTokenType::COMMENT,
                                ],
                                token_modifiers: vec![],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(false),
                            ..Default::default()
                        },
                    ),
                ),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.клиент
            .log_message(MessageType::INFO, "ЗИП-LSP сервер запущен")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let док = разобрать_документ(params.text_document.text, params.text_document.version);
        опубликовать_диагностику(&self.клиент, uri.clone(), &док).await;
        self.документы.write().await.insert(uri, док);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(изм) = params.content_changes.into_iter().last() {
            let док = разобрать_документ(изм.text, params.text_document.version);
            опубликовать_диагностику(&self.клиент, uri.clone(), &док).await;
            self.документы.write().await.insert(uri, док);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.документы.write().await.remove(&params.text_document.uri);
        self.клиент
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let mut элементов = Vec::new();

        let ключевые = [
            ("если", "Условный оператор"),
            ("иначе", "Альтернативная ветка"),
            ("пока", "Цикл с предусловием"),
            ("для", "Цикл по коллекции"),
            ("в", "Вхождение в цикле для"),
            ("возврат", "Возврат из функции"),
            ("прервать", "Прервать цикл"),
            ("продолжить", "Продолжить цикл"),
            ("функция", "Объявление функции"),
            ("тип", "Объявление типа"),
            ("модуль", "Объявление модуля"),
            ("импорт", "Импорт модуля"),
            ("истина", "Истина"),
            ("ложь", "Ложь"),
            ("и", "Логическое И"),
            ("или", "Логическое ИЛИ"),
            ("не", "Логическое НЕ"),
        ];
        for (слово, описание) in ключевые {
            элементов.push(CompletionItem {
                label: слово.into(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(описание.into()),
                insert_text: Some(слово.into()),
                ..Default::default()
            });
        }

        let глаголы = [
            ("печать_строки", "Вывод строки", "печать_строки(\"$1\")"),
            ("печать_число", "Вывод числа", "печать_число($1)"),
            ("печать", "Вывод без \\n", "печать($1)"),
            ("САПАР", "Глагол счёта", "САПАР($1)"),
            ("ЧАТАВ", "Фиксация результата", "ЧАТАВ"),
            ("ЖАВАТ", "Асинхронный поток", "ЖАВАТ~"),
            ("создать_тензор", "Создать тензор", "создать_тензор($1)"),
            ("прямой_проход", "Прямой проход нейросети", "прямой_проход($1)"),
            ("получить", "HTTP/HTTPS GET", "получить($1)"),
            ("хеш_sha256_hex", "SHA-256 хеш", "хеш_sha256_hex($1)"),
        ];
        for (имя, описание, snippet) in глаголы {
            элементов.push(CompletionItem {
                label: имя.into(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(описание.into()),
                insert_text: Some(snippet.into()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        let операторы = [(">>", "Импульс"), ("||", "Хор"), ("!", "Фиксация"), ("~", "Поток")];
        for (оп, описание) in операторы {
            элементов.push(CompletionItem {
                label: оп.into(),
                kind: Some(CompletionItemKind::OPERATOR),
                detail: Some(описание.into()),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(элементов)))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let позиция = params.text_document_position_params.position;
        let документы = self.документы.read().await;
        let док = match документы.get(uri) {
            Some(д) => д,
            None => return Ok(None),
        };
        let Some((слово, range)) = слово_в_позиции(&док.текст, позиция) else {
            return Ok(None);
        };
        let содержимое = hover_для(&слово).unwrap_or_else(|| {
            format!("**`{слово}`** — идентификатор ZIP")
        });
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: содержимое,
            }),
            range: Some(range),
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let позиция = params.text_document_position_params.position;
        let документы = self.документы.read().await;
        let док = match документы.get(&uri) {
            Some(д) => д,
            None => return Ok(None),
        };
        let Some((слово, _)) = слово_в_позиции(&док.текст, позиция) else {
            return Ok(None);
        };
        // Ищем первое вхождение объявления в токенах (грубо)
        for т in &док.токены {
            let имя = match &т.токен {
                Токен::Слово(s) | Токен::Шаблон(s) | Токен::Корень(s) => s.as_str(),
                _ => continue,
            };
            if имя == слово {
                if let Some(дерево) = &док.дерево {
                    if найти_определение(дерево, &слово) {
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                            uri: uri.clone(),
                            range: диапазон_точки(т.строка, т.столбец, длина_токена(&т.токен)),
                        })));
                    }
                }
            }
        }
        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let документы = self.документы.read().await;
        let док = match документы.get(&params.text_document.uri) {
            Some(д) => д,
            None => return Ok(None),
        };
        let mut символы = Vec::new();
        if let Some(д) = &док.дерево {
            собрать_символы(д, &mut символы);
        }
        Ok(Some(DocumentSymbolResponse::Nested(символы)))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> LspResult<Option<SemanticTokensResult>> {
        let документы = self.документы.read().await;
        let док = match документы.get(&params.text_document.uri) {
            Some(д) => д,
            None => return Ok(None),
        };

        let mut data = Vec::new();
        let mut prev_line = 0u32;
        let mut prev_start = 0u32;

        for т in &док.токены {
            let Some(ty) = тип_семантики(&т.токен) else {
                continue;
            };
            let line = т.строка.saturating_sub(1) as u32;
            let start = т.столбец.saturating_sub(1) as u32;
            let len = длина_токена(&т.токен).max(1);
            let delta_line = line.saturating_sub(prev_line);
            let delta_start = if delta_line == 0 {
                start.saturating_sub(prev_start)
            } else {
                start
            };
            data.push(SemanticToken {
                delta_line,
                delta_start,
                length: len,
                token_type: ty,
                token_modifiers_bitset: 0,
            });
            prev_line = line;
            prev_start = start;
        }

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data,
        })))
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> LspResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let документы = self.документы.read().await;
        let док = match документы.get(&uri) {
            Some(д) => д,
            None => return Ok(None),
        };

        let линия = params.range.start.line as usize;
        let строка = док.текст.lines().nth(линия).unwrap_or("");
        let мутации = латинские_замены_в_строке(строка, линия as u32);
        if мутации.is_empty() {
            return Ok(None);
        }

        let mut действия = Vec::new();
        for (range, оригинал, замена) in мутации {
            let mut changes = HashMap::new();
            changes.insert(
                uri.clone(),
                vec![TextEdit {
                    range,
                    new_text: замена.clone(),
                }],
            );
            действия.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!(
                    "Заменить «{оригинал}» → «{замена}» (латиница запрещена в ZIP)"
                ),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(params.context.diagnostics.clone()),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }),
                is_preferred: Some(true),
                ..Default::default()
            }));
        }

        // Одна правка: заменить все латинские идентификаторы на строке
        if действия.len() > 1 {
            let все = латинские_замены_в_строке(строка, линия as u32);
            let edits: Vec<TextEdit> = все
                .into_iter()
                .map(|(range, _, замена)| TextEdit {
                    range,
                    new_text: замена,
                })
                .collect();
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), edits);
            действия.insert(
                0,
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Заменить всю латиницу на строке кириллицей".into(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    is_preferred: Some(true),
                    ..Default::default()
                }),
            );
        }

        Ok(Some(действия))
    }
}

fn латиница_в_кириллицу(символ: char) -> Option<char> {
    let нижний = символ.to_ascii_lowercase();
    let кир = match нижний {
        'a' => 'а',
        'b' => 'б',
        'c' => 'ц',
        'd' => 'д',
        'e' => 'е',
        'f' => 'ф',
        'g' => 'г',
        'h' => 'х',
        'i' => 'и',
        'j' => 'й',
        'k' => 'к',
        'l' => 'л',
        'm' => 'м',
        'n' => 'н',
        'o' => 'о',
        'p' => 'п',
        'q' => 'к',
        'r' => 'р',
        's' => 'с',
        't' => 'т',
        'u' => 'у',
        'v' => 'в',
        'w' => 'в',
        'x' => 'х',
        'y' => 'у',
        'z' => 'з',
        _ => return None,
    };
    if символ.is_ascii_uppercase() {
        кир.to_uppercase().next()
    } else {
        Some(кир)
    }
}

/// Находит подряд идущие латинские идентификаторы на строке → (range, original, replacement).
fn латинские_замены_в_строке(строка: &str, line: u32) -> Vec<(Range, String, String)> {
    let mut out = Vec::new();
    let chars: Vec<(usize, char)> = строка.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (byte_start, ch) = chars[i];
        if !ch.is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        // Не трогаем содержимое строк "..."
        let перед = &строка[..byte_start];
        let кавычек = перед.matches('"').count();
        if кавычек % 2 == 1 {
            i += 1;
            continue;
        }

        let start_col = строка[..byte_start].chars().count() as u32;
        let mut оригинал = String::new();
        let mut замена = String::new();
        while i < chars.len() {
            let (_, c) = chars[i];
            if c.is_ascii_alphabetic() || c == '_' || c.is_ascii_digit() {
                оригинал.push(c);
                if let Some(к) = латиница_в_кириллицу(c) {
                    замена.push(к);
                } else if c == '_' || c.is_ascii_digit() {
                    замена.push(c);
                } else {
                    break;
                }
                i += 1;
            } else {
                break;
            }
        }
        if оригинал.is_empty() || оригинал == замена {
            continue;
        }
        let end_col = start_col + оригинал.chars().count() as u32;
        out.push((
            Range {
                start: Position {
                    line,
                    character: start_col,
                },
                end: Position {
                    line,
                    character: end_col,
                },
            },
            оригинал,
            замена,
        ));
    }
    out
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|клиент| ЗипLsp {
        клиент,
        документы: Arc::new(RwLock::new(HashMap::new())),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
