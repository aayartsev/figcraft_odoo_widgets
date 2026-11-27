# figcraft_odoo_widgets

Библиотека UI-виджетов **Odoo 20 Community** для макетов в FigCraft.

Каждый виджет — **нативный JSON-документ FigCraft**: открывается в FigCraft без сборки. Метаданные для будущей склейки экранов — в [`catalog.json`](catalog.json).

Спека: [docs/superpowers/specs/2026-11-26-native-widget-storage-design.md](docs/superpowers/specs/2026-11-26-native-widget-storage-design.md).

## Структура

```
widgets/             # нативные JSON FigCraft (один файл = один виджет)
catalog.json         # обёртка: class, category, slots, size, …
schemas/             # schema каталога
figcraft_build/      # validate
demos/               # устаревший справочный compose (вне нового формата)
dist/                # локальные артефакты (gitignore)
```

## Как открыть виджет

В FigCraft: File → Open (или MCP `open_file`) на файл вроде `widgets/fields/date_time_field.json`.  
Конвертация в бинарный `.figcraft` **не нужна** и в пайплайне не используется.

## Проверка каталога

```bash
cargo run -p figcraft_build -- validate
```

## Правила

1. Имя файла = `class` в **snake_case** (`Navbar` → `navbar.json`)
2. В файле макета — только нативный документ FigCraft
3. Описание, слоты, размер, варианты — в записи `catalog.json`
4. Точки монтирования — фреймы `slot:<name>` в макете
5. В репозитории и пайплайне — **только текстовые JSON**, без бинарных `.figcraft`

## Категории

| Категория | Зачем |
|-----------|--------|
| `shell` | Каркас приложения (navbar + область действия) |
| `navbar_widgets` | Элементы верхней панели и systray |
| `menu` | Содержимое боковой панели AppsMenu |
| `views` | list / form / kanban, control panel, search, calendar / graph / pivot / activity / hierarchy |
| `fields` | Поля форм |
| `discussion` | UI Discuss / сообщений |
| `custom` | Разовые виджеты под проект |

## Визуальный ориентир

Odoo 20 **Community**: бренд `#71639E`, фон `#F8F9FA`, Inter в макетах FigCraft. Узнаваемый стиль, не пиксель-перфект.

## Демо

[`demos/sale_order_form.json`](demos/sale_order_form.json) — нативный демо-экран **sale.order** (Navbar + CP + Form + StatusBar + строки + итоги) для обсуждения с клиентом.  
Сборка полных экранов — правило [`.cursor/rules/figcraft-form-demo-compose.mdc`](.cursor/rules/figcraft-form-demo-compose.mdc).  
[`demos/demo_shell_empty_action.json`](demos/demo_shell_empty_action.json) — **устаревший** справочный compose (старый гибридный DSL); `validate` его не проверяет.
