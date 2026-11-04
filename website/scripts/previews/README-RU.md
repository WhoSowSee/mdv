# Превью документации

[English](README.md)

Записанный вывод mdv хранится в `src/data/render-previews.json` и
`src/data/interactive-previews.json`. Обновляйте его после изменения рендеринга или интерактивных режимов.

## Генерация и проверка

Нужны Bun 1.4.4 или новее и Python 3.11 или новее. Начните из корня репозитория.

```sh
cargo build --release
cd website
python -m pip install -r scripts/preview-requirements.txt
bun run update-previews
bun run update-previews --check
```

`update-previews` принимает необязательный путь к бинарному файлу mdv, версия
которого должна совпадать с `Cargo.toml`. `--check` сравнивает оба снимка без перезаписи.
Захват игнорирует пользовательскую конфигурацию и настройки цветов окружения.

## Шрифты

Пересоздавайте шрифты превью после изменения текстовых шрифтов или добавления символов в снимки.
Ссылки на исходники и контрольные суммы указаны в [symbol-sources.json](symbol-sources.json).
Выполните из `website`, передав пути к этим исходным шрифтам.

```sh
python -m pip install -r scripts/font-requirements.txt
python scripts/build-preview-symbols.py JetBrainsMonoNerdFont-Regular.ttf NotoSansSymbols2-Regular.ttf
```
