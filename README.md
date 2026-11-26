# figcraft_odoo_widgets

Библиотека UI-виджетов **Odoo 20 Web Client** для макетов в FigCraft.

Каждый виджет — один JSON: метаданные для сборки страниц и визуальный фрагмент FigCraft. Из них удобно собирать демо-экраны и обсуждать интерфейс с клиентами.

## Структура

```
widgets/
  shell/             # каркас: WebClientShell, Navbar
  navbar_widgets/    # apps-меню, systray, пользователь, сообщения
  menu/              # RootMenu, MenuItem
  views/             # control panel, list / form / kanban
  fields/            # стандартные поля форм
  custom_views/      # calendar, graph, pivot, activity (+ _template.json)
  discussion/        # Discuss: sidebar, thread, message, composer
  custom/            # виджеты под конкретный проект (+ _template.json)
demos/               # собранные демо-страницы FigCraft
schemas/             # JSON Schema для файлов виджетов
catalog.json         # каталог всех виджетов
```

## Правила файла виджета

1. Один класс виджета → один JSON-файл
2. Имя файла = `class` в **snake_case** (`Navbar` → `navbar.json`)
3. Гибридный формат: метаданные (`slots`, `size`, …) + фрагмент `figcraft.root`
4. Точки монтирования слотов — фреймы с именем `slot:<name>`
5. Шаблоны `_template.json` в каталог не входят — скопируйте и переименуйте под новый виджет

Подробности: [docs/superpowers/specs/2026-11-26-odoo-widget-format-design.md](docs/superpowers/specs/2026-11-26-odoo-widget-format-design.md).

## Категории

| Категория | Зачем |
|-----------|--------|
| `shell` | Каркас приложения |
| `navbar_widgets` | Элементы systray и apps-меню |
| `menu` | Боковая навигация |
| `views` | Стандартные list / form / kanban и control panel |
| `fields` | Поля форм |
| `custom_views` | Другие режимы вида: calendar, graph, pivot, activity |
| `discussion` | UI Discuss / сообщений (отдельно от systray) |
| `custom` | Разовые виджеты под проект |

## Визуальный ориентир

Типичный chrome Odoo Web Client: бренд `#714B67`, фон `#F9F9F9`, акцент Discuss `#008784`. Это узнаваемый стиль, а не пиксель-перфект с живой базы.

## Демо

[`demos/demo_shell_empty_action.json`](demos/demo_shell_empty_action.json) — shell + navbar + root menu + control panel + пустая область view.

[`demos/demo_contacts_list_kanban_form.figcraft`](demos/demo_contacts_list_kanban_form.figcraft) — Contacts в трёх режимах (List / Kanban / Form). Рядом лежит [`demo_contacts_list_kanban_form.json`](demos/demo_contacts_list_kanban_form.json) — собранное дерево слотов (исходник композиции); в FigCraft открывайте `.figcraft`.

## С чего начать

1. Выберите виджеты в [`catalog.json`](catalog.json)
2. Откройте или импортируйте фрагменты `figcraft` в FigCraft
3. Вставьте дочерние виджеты во фреймы `slot:*`
4. Для своих кусков скопируйте `widgets/custom/_template.json` или `widgets/custom_views/_template.json`
