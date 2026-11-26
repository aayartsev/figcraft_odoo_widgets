# figcraft_odoo_widgets

Библиотека UI-виджетов **Odoo 20 Community** для макетов в FigCraft.

Каждый виджет — один JSON: метаданные для сборки страниц и визуальный фрагмент FigCraft. Из них удобно собирать демо-экраны и обсуждать интерфейс с клиентами.

## Структура

```
widgets/
  shell/             # каркас: WebClientShell, Navbar
  navbar_widgets/    # apps-меню, разделы, systray, пользователь, сообщения, компания
  menu/              # содержимое боковой панели AppsMenu (RootMenu, MenuItem)
  views/             # control panel, list / form / kanban
  fields/            # стандартные поля форм
  custom_views/      # calendar, graph, pivot, activity (+ _template.json)
  discussion/        # Discuss: sidebar, thread, message, composer
  custom/            # виджеты под конкретный проект (+ _template.json)
demos/               # справочные compose (не все собираются билдером)
schemas/             # JSON Schema для файлов виджетов
catalog.json         # каталог всех виджетов
figcraft_build/      # Rust-утилита: validate + atlas → .figcraft
dist/                # собранные .figcraft (gitignore)
```

## Сборка атласа

Источник правды — JSON. Файл для FigCraft собирает утилита:

```bash
cargo run -p figcraft_build -- validate
cargo run -p figcraft_build -- atlas
```

Откройте `dist/atlas_all_widgets.figcraft` в FigCraft **0.18.1**. Подробности: [`figcraft_build/README.md`](figcraft_build/README.md) и [спека билдера](docs/superpowers/specs/2026-11-26-figcraft-build-tool-design.md).

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
| `shell` | Каркас приложения (navbar + область действия) |
| `navbar_widgets` | Элементы верхней панели и systray |
| `menu` | Содержимое боковой панели AppsMenu (не постоянное левое меню) |
| `views` | Стандартные list / form / kanban и control panel |
| `fields` | Поля форм |
| `custom_views` | Другие режимы вида: calendar, graph, pivot, activity |
| `discussion` | UI Discuss / сообщений (отдельно от systray) |
| `custom` | Разовые виджеты под проект |

## Визуальный ориентир

Odoo 20 **Community**: бренд `#71639E`, фон `#F8F9FA`, основные кнопки того же бренда, акцент Discuss `#017E84`, ссылки формы / many2one `#66598F`, badge navbar `#28A745`. В макетах FigCraft шрифт **Inter**. Это узнаваемый стиль, а не пиксель-перфект с живой базы.

## Демо

[`demos/demo_shell_empty_action.json`](demos/demo_shell_empty_action.json) — справочный compose: shell + navbar (apps / разделы / systray) + control panel + пустая область view. Без постоянного левого меню. Пока **не** собирается `figcraft_build` (билдер v1 делает только атлас виджетов).

## С чего начать

1. Выберите виджеты в [`catalog.json`](catalog.json)
2. Соберите атлас (`cargo run -p figcraft_build -- atlas`) и откройте в FigCraft
3. Для своих кусков скопируйте `widgets/custom/_template.json` или `widgets/custom_views/_template.json`
