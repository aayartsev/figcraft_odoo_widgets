# figcraft_build

Проверяет каталог виджетов Odoo: `catalog.json` + нативные JSON FigCraft в `widgets/`.

Формат: [docs/superpowers/specs/2026-11-26-native-widget-storage-design.md](../docs/superpowers/specs/2026-11-26-native-widget-storage-design.md).

## Требования

- Rust **1.95+** ([`rust-toolchain.toml`](../rust-toolchain.toml))
- Сеть при первой сборке: git-зависимости FigCraft (pin `4acc293…`, приложение **0.18.1**)

## Команды

```bash
cargo run -p figcraft_build -- validate
```

Флаги: `--root`, `--catalog`.

Бинарный `.figcraft` утилита не пишет.

## Тесты

```bash
cargo test -p figcraft_build
```
