# Формат гибридного JSON-виджета Odoo

> **Устарела.** Актуальная спека: [2026-11-26-native-widget-storage-design.md](2026-11-26-native-widget-storage-design.md) (нативный JSON FigCraft + каталог-обёртка, без бинарных `.figcraft`).

**Дата:** 2026-11-26  
**Целевая версия Odoo:** 20 **Community** (типичный вид Web Client)  
**Статус:** устарела (заменена native storage)

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
| `navbar_widgets` | `widgets/navbar_widgets/` | Куски apps-меню, разделы, systray |
| `menu` | `widgets/menu/` | Содержимое боковой панели AppsMenu (sidebar), не постоянное левое меню |
| `views` | `widgets/views/` | Все представления action: list / form / kanban, CP, search, calendar / graph / pivot / activity / hierarchy |
| `fields` | `widgets/fields/` | Виджеты полей форм и списков |
| `discussion` | `widgets/discussion/` | UI Discuss / сообщений |
| `custom` | `widgets/custom/` | Виджеты под конкретный проект |

## Схема документа

Машиночитаемая схема гибрида удалена; актуальная схема каталога: [`schemas/catalog.schema.json`](../../../schemas/catalog.schema.json)

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
{ "name": "systray", "accepts": ["NavbarSystrayItem", "NavbarUserMenu", "NavbarMessagingMenu", "NavbarSwitchCompanyMenu"] }
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
  "props": { "fill": "#71639E", "layout": "horizontal", "gap": 8, "padding": [0, 12, 0, 12] },
  "children": []
}
```

Допустимые `type` совпадают с FigCraft: `frame`, `rectangle`, `ellipse`, `text`, `line`, `vector`, `component`, `section`, `slice`, `polygon`, `star`, `arrow`.

`props` — те же ключи, что у FigCraft `set_properties` (fill, layout, gap, padding, fontSize, …).

У текстовых узлов символы лежат в `text`.

Плейсхолдеры слотов — фреймы с именем `slot:<slotName>` (например `slot:systray`), чтобы сборщик находил точки монтирования.

Шрифт в макетах FigCraft: **`Inter`**. В живом Odoo — system UI stack; в JSON виджетов фиксируем Inter как стабильную замену.

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

## Визуальный язык (Odoo 20 Community)

Не пиксель-перфект с живой БД. Ориентир — **Community** (`$o-community-color`). Все hex в файлах — верхний регистр.

| token | значение | где |
|-------|----------|-----|
| `brand` | `#71639E` | Navbar, основные акценты |
| `brandDark` | `#5C5282` | Библиотечный hover / active в navbar (~10% затемнение бренда; в SCSS Odoo отдельного имени нет) |
| `surface` | `#FFFFFF` | Карточки, панели |
| `canvas` | `#F8F9FA` | Фон приложения (`$o-gray-100`) |
| `border` | `#DEE2E6` | Разделители, инпуты |
| `text` | `#212529` | Основной текст |
| `muted` | `#6C757D` | Вторичные подписи |
| `primaryBtn` | `#71639E` | Основные кнопки (в Community `$o-action` = бренд) |
| `link` / Discuss | `#017E84` | Акцент Discuss / messaging (`$o-enterprise-action-color`) |
| `formLink` | `#66598F` | Ссылки формы / many2one (`$link-color` = darken brand 5%) |
| `navbarBadge` | `#28A745` | Badge в верхней панели (`$o-success`) |

Каркас приложения: **верхняя панель (`Navbar`) + область действия** — без постоянного левого меню. `RootMenu` / `MenuItem` — содержимое боковой панели AppsMenu, категория `menu` сохраняется.

Размер shell по умолчанию: **1366×768**. Высота navbar: **46px**.

## Правила сборки

1. Родитель монтирует детей только в объявленные `slots` / фреймы `slot:*`
2. `size` ребёнка должен помещаться в слот; ширину можно растягивать до `fill`, если у родителя auto-layout
3. Демо-страницы лежат в `demos/` как собранные документы FigCraft (целый файл, не гибридные метаданные)

## Виды и custom widgets

- **views** — все режимы action view в одной папке: list / form / kanban, control panel, search, calendar / graph / pivot / activity / hierarchy, …
- **custom** — разовые виджеты проекта (KPI, особые блоки форм), которых нет в ядре Odoo

## Discussion

UI Discuss — отдельная категория (`discussion`), а не часть navbar systray. Меню сообщений в navbar может концептуально вести к виджетам Discuss, но само остаётся в `navbar_widgets`.
