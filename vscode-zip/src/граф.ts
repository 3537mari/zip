/**
 * Типы для команды визуализации графа (Идея 6).
 * Реализация — `граф.js` (расширение без tsc-сборки).
 */

export interface Ассоциация {
  связанное: string;
  тип: string;
  иконка: string;
}

export interface СнимокАссоциаций {
  слово: string;
  описание?: string;
  тип?: string;
  категории?: string[];
  ассоциации: Ассоциация[];
}

export interface УзелГрафа {
  имя: string;
  тип: string;
  описание: string;
  категории: string[];
}

export interface РеброГрафа {
  от: string;
  к: string;
  тип: string;
  иконка: string;
}

export interface СнимокГрафа {
  узлы: УзелГрафа[];
  рёбра: РеброГрафа[];
}
