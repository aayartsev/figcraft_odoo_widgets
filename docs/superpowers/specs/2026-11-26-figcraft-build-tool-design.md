> **Устарела по части atlas/`.figcraft`.** Актуальная утилита — только `validate` над нативными JSON; см. [2026-11-26-native-widget-storage-design.md](2026-11-26-native-widget-storage-design.md).

# Утилита сборки FigCraft из JSON-виджетов

**Дата:** 2026-11-26  
**Статус:** утверждено к реализации (v1)

## Зачем

JSON в `widgets/` — источник правды. Файл `.figcraft` собирается алгоритмически, чтобы открыть все виджеты в FigCraft одним атласом и сверять внешний вид. Экранные формы — следующий этап.

## Где лежит код

Каталог [`figcraft_build/`](../../../figcraft_build/) в корне репозитория. Workspace Cargo: [`Cargo.toml`](../../../Cargo.toml).

## Зависимости FigCraft

Крейты `figcraft-doc` и `figcraft-format` с git pin на commit приложения FigCraft **0.18.1** (`4acc293c2ec941ac1400837ee7e3ab51780399f8`). Не crates.io 0.5.1.

Toolchain: `rust-toolchain.toml` → channel `1.95` (MSRV upstream).

## Команды v1

| команда | результат |
|---------|-----------|
| `validate` | schema + инварианты по всему `catalog.json` |
| `atlas` | `dist/atlas_all_widgets.figcraft` |

Без `normalize` / `figcraft-engine` в v1: координаты и размеры из JSON как записано.

## Конвертер

Типы: `frame`, `rectangle`, `ellipse`, `text`.  
Разрешённые `props`: только ключи, которые есть в текущем catalog (`fill`, `layout`, `gap`, `padding`, …). Иной тип или ключ — ошибка.

## Атлас

Порядок секций категорий — как в спеке формата виджетов (`shell` → … → `custom`). Пустые категории не рисуются. Внутри секции — порядок записей в `catalog.json`. Ряды с зазором 40px, перенос при ширине > 2400px.

## Проверка

```bash
cargo test -p figcraft_build
cargo run -p figcraft_build -- validate
cargo run -p figcraft_build -- atlas
```

Открыть `dist/atlas_all_widgets.figcraft` в FigCraft 0.18.1.

## Вне v1

Сборка экранов и монтаж в `slot:*`, `normalize`, thumbnail, типы слоёв вне catalog.
