# figcraft_build

Утилита: проверяет JSON-виджеты Odoo и собирает файл атласа `.figcraft`.

## Требования

- Rust **1.95+** (см. [`rust-toolchain.toml`](../rust-toolchain.toml) в корне репо)
- Сеть при первой сборке: git-зависимости FigCraft

Pin FigCraft: commit `4acc293c2ec941ac1400837ee7e3ab51780399f8` (приложение **0.18.1**).

В v1 нет `figcraft-engine::normalize`: после сборки атласа размеры и позиции автолейаут-фреймов считаются упрощённо из размеров детей, чтобы файл было видно в FigCraft.

## Команды

Из корня репозитория:

```bash
cargo run -p figcraft_build -- validate
cargo run -p figcraft_build -- atlas
cargo run -p figcraft_build -- widget Navbar
```

Артефакты пишутся в `dist/` (в `.gitignore`). Один виджет → `dist/<snake_case>.figcraft` (например `dist/navbar.figcraft`).

Флаги:

- `--root <path>` — корень с `catalog.json` / `widgets/`
- `--catalog <path>` — путь к каталогу
- `atlas --out <path>` / `widget --out <path>` — куда писать `.figcraft`

## Тесты

```bash
cargo test -p figcraft_build
```
