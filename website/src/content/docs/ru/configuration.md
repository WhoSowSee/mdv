---
title: "Конфигурация"
description: "Поиск конфигурации, приоритет источников и структурированные параметры YAML."
slug: "configuration"
group: "Reference"
order: 40
---

## Настройки и приоритет

mdv объединяет настройки из нескольких источников в порядке уменьшения приоритета:

1. Параметры командной строки (максимальный приоритет).
2. `MDV_COLOR`, только для настройки `color`.
3. Пресет, выбранный через `--preset`.
4. Конфигурация из `--config-file`, `MDV_CONFIG_PATH` или пользовательского каталога.
5. Встроенные значения по умолчанию.

Создать конфиг по умолчанию можно командой `mdv --init-config`. Укажите каталог (`mdv --init-config custom/dir`), используйте `--config-file <CONFIG_DIR>` или задайте `MDV_CONFIG_PATH`, чтобы записать его в другой путь.

Файлы конфигурации поддерживают YAML (`.yaml` или `.yml`). Все настройки и примеры приведены в [шаблоне конфигурации](../configuration-reference/):

```yaml
# docs/examples/config.yaml
theme: "terminal"
code_theme: null
inline_style: {}
wrap: "char"
table_wrap: "fit"
table_borders: false
heading_layout: "level"
smart_indent: false
block_spacing: null
code_wrap_indent: "double"
link_style: "clickable"
link_overflow: "wrap"
```

## Пути конфигурации

mdv проверяет `config.yaml` перед `config.yml`. Кандидаты из `--config-file DIR` идут перед `MDV_CONFIG_PATH`; если оба источника отсутствуют, используется пользовательский каталог настроек ОС. В Windows это `%USERPROFILE%\.config\mdv`, в Linux `$XDG_CONFIG_HOME/mdv` или `~/.config/mdv`, в macOS `~/Library/Application Support/mdv`.

Загружается первый существующий файл. Ошибки YAML или настроек останавливают запуск без перехода к следующему файлу. `--init-config` создаёт новый файл и отказывается перезаписывать существующий. `--no-config` пропускает основной файл, сохраняя выбранный каталог для тем и пресетов.

```sh
mdv --init-config ./mdv-config
mdv --config-file ./mdv-config README.md
mdv --no-config README.md
```

## Правила объединения и очистки

Файлы конфигурации и пресетов принимают структурированные YAML-mapping для `custom_theme`, `custom_code_theme`, `block_spacing`, `custom_callout`, `custom_code_block`, `custom_checkbox`, `custom_list`, `callout_style`. Компактные строки CLI также допустимы в YAML.

| Настройка | Объединение |
| --- | --- |
| Обычные значения | Явно заданное значение большего приоритета заменяет предыдущее, включая `false`, значения по умолчанию и `null`. |
| `custom_theme`, `custom_code_theme` | Цвета объединяются по полям. Внутри источника порядок цветов синтаксиса: `custom_theme`, `code_theme`, `custom_code_theme`. Более высокий источник важнее даже между разными параметрами переопределения. |
| `inline_style` | Объединение по элементу и свойству оформления. Явный `false` отключает атрибут. |
| `custom_callout`, `custom_code_block`, `custom_checkbox`, `custom_list` | Объединение по записи и свойству; пропущенные свойства сохраняют значения и приоритет источника. |
| `block_spacing` | Mapping конфигурации/пресета заменяет предыдущий; CLI объединяет только явно указанные элементы и стороны. |

В пользовательских записях `null` очищает свойство или запись, корневые `null` и `{}` очищают всю настройку. Переданный `aliases` заменяет весь список, `aliases: []` очищает его. Структурированный `callout_style` принимает `style`, `show_icons`, `show_simple_icons`, `fold_icons`, `label_inside`, `uppercase`.

```yaml
custom_callout:
  note:
    color: red
  tip: null
custom_code_block:
  python:
    aliases: []
inline_style:
  code:
    backticks: false
```

`MDV_COLOR` важнее пресета и конфигурации, но явный `--color` важнее переменной. Значения окружения проверяются до переопределений CLI. Неизвестные структурированные свойства, дубликаты и некорректные значения приводят к ошибкам. [Все ключи и значения по умолчанию](../configuration-reference/), [наследование цветов](../theme-reference/) и [переменные окружения](../environment/) описаны на сайте.
