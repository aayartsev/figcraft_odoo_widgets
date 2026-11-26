# Формат гибридного JSON-виджета Odoo

**Дата:** 2026-11-26  
**Целевая версия Odoo:** 20 (типичный вид Web Client)  
**Статус:** утверждено к реализации

## Зачем это

Каждый переиспользуемый кусок интерфейса Odoo — один гибридный JSON: метаданные для сборки страниц плюс фрагмент FigCraft. Из таких файлов собираются демо для обсуждения с клиентами.

## Именование файлов

- Имя файла = `class` виджета в snake_case + `.json`
- Примеры: `Navbar` → `navbar.json`, `WebClientShell` → `web_client_shell.json`
- Один класс — один файл; визуальные варианты живут в `variants`, а не в отдельных файлах

## Категории

| category | путь | роль |
|----------|------|------|
| `shell` | `widgets/shell/` | Каркас приложения (web client, navbar) |
| `navbar_widgets` | `widgets/navbar_widgets/` | Куски apps-меню и systray |
| `menu` | `widgets/menu/` | Корневое меню и пункты |
| `views` | `widgets/views/` | Стандартные виды + control panel |
| `fields` | `widgets/fields/` | Виджеты полей форм и списков |
| `custom_views` | `widgets/custom_views/` | Нестандартные виды (calendar, graph, pivot, …) |
| `discussion` | `widgets/discussion/` | UI Discuss / сообщений |
| `custom` | `widgets/custom/` | Виджеты под конкретный проект |

## Схема документа

Машиночитаемая схема: [`schemas/widget.schema.json`](../../../schemas/widget.schema.json)

Обязательные поля верхнего уровня:

- `schemaVersion` (число, сейчас `1`)
- `class` (имя класса виджета в PascalCase)
- `odooVersion` (строка, например `"20"`)
- `category` (одна из категорий выше)
- `description` (короткое описание по-русски)
- `size` (`width`, `height` в px при 100% зуме)
- `slots` (массив; пустой, если виджет листовой)
- `variants` (массив строк; минимум `"default"`)
- `figcraft` (`kind: "fragment"`, дерево `root`)

Необязательные:

- `slotAnchors` — словарь: имя слота → имя слоя внутри `figcraft.root`, куда монтируются дети
- `tags` — свободные теги для поиска

### Объект слота

```json
{ "name": "systray", "accepts": ["NavbarSystrayItem", "NavbarUserMenu"] }
```

`accepts` — какие классы виджетов можно положить в слот. Пустой `accepts` значит «любой виджет».

### Фрагмент FigCraft

`figcraft.root` — рекурсивный узел:

```json
{
  "type": "frame",
  "name": "Navbar",
  "x": 0,
  "y": 0,
  "width": 1366,
  "height": 46,
  "props": { "fill": "#714B67", "layout": "horizontal", "gap": 8, "padding": [0, 12, 0, 12] },
  "children": []
}
```

Допустимые `type` совпадают с FigCraft: `frame`, `rectangle`, `ellipse`, `text`, `line`, `vector`, `component`, `section`, `slice`, `polygon`, `star`, `arrow`.

`props` — те же ключи, что у FigCraft `set_properties` (fill, layout, gap, padding, fontSize, …).

У текстовых узлов символы лежат в `text`.

Плейсхолдеры слотов — фреймы с именем `slot:<slotName>` (например `slot:systray`), чтобы сборщик находил точки монтирования.

## Каталог

[`catalog.json`](../../../catalog.json) индексирует все виджеты:

```json
{
  "schemaVersion": 1,
  "odooVersion": "20",
  "widgets": [
    { "class": "Navbar", "category": "shell", "path": "widgets/shell/navbar.json" }
  ]
}
```

## Визуальный язык (типичный Odoo 20 Web Client)

Не пиксель-перфект с живой БД. Общие токены:

| token | значение | где |
|-------|----------|-----|
| `brand` | `#714B67` | Navbar, основные акценты |
| `brandDark` | `#5D3E56` | Hover / active в navbar |
| `surface` | `#FFFFFF` | Карточки, панели |
| `canvas` | `#F9F9F9` | Фон приложения |
| `border` | `#DEE2E6` | Разделители, инпуты |
| `text` | `#212529` | Основной текст |
| `muted` | `#6C757D` | Вторичные подписи |
| `primaryBtn` | `#714B67` | Основные кнопки |
| `link` | `#008784` | Ссылки / акцент Discuss |

Размер shell по умолчанию: **1366×768**.

## Правила сборки

1. Родитель монтирует детей только в объявленные `slots` / фреймы `slot:*`
2. `size` ребёнка должен помещаться в слот; ширину можно растягивать до `fill`, если у родителя auto-layout
3. Демо-страницы лежат в `demos/` как собранные документы FigCraft (целый файл, не гибридные метаданные)

## Custom views и custom widgets

- **custom_views** — альтернативные режимы action view (calendar, graph, pivot, activity, map, …)
- **custom** — разовые виджеты проекта (KPI, особые блоки форм), которых нет в ядре Odoo

## Discussion

UI Discuss — отдельная категория (`discussion`), а не часть navbar systray. Меню сообщений в navbar может концептуально вести к виджетам Discuss, но само остаётся в `navbar_widgets`.
