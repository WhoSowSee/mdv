---
title: "Списки и чекбоксы"
description: "Состояния задач, маркеры вложенных списков, определения и свои цвета."
slug: "lists"
group: "Formatting"
order: 24
---

## Чекбоксы списков задач

- `-x, --checkbox-style <square|circle>` - рендерит чекбоксы task-list как иконки Nerd Font вместо стандартных `[ ]` / `[x]`. Требует шрифт Nerd Font в терминале.
- `--custom-checkbox <PAIRS>` - переопределяет встроенные иконки чекбоксов или добавляет новые состояния (только с `--checkbox-style`). Каждая разделённая точкой с запятой пара задаётся как `<char>:<icon>[:<color>]` или `<char>:<color>`. Форма только с цветом сохраняет существующую иконку, а для нового состояния использует стандартную иконку неотмеченного чекбокса. Цвета принимают именованные, hex, rgb и `ansi(N)`.
  - Переопределение:  `--custom-checkbox ' :!'`         заменить иконку неотмеченного
  - Добавление:       `--custom-checkbox '*:*'`         добавить состояние `[*]`
  - Цвет:             `--custom-checkbox ' :!:yellow'`  или `#ff0000`, `128,1,1`, `ansi(200)`
  - Только цвет: `--custom-checkbox '?:red'` сохраняет значок Nerd Font для состояния `?` и задаёт красный цвет.
  - Новое состояние, только цвет: `--custom-checkbox '*:yellow'` использует стандартный значок неотмеченного чекбокса и жёлтый цвет.

<div data-mdv-preview="checkbox-style"></div>

<div data-mdv-preview="custom-checkbox"></div>

## Маркеры списков и определения

- `-L, --list-style <стиль>` - заменяет стандартные маркеры `-` неупорядоченных списков на набор иконок по уровням. Значение обязательно и задаётся в формате `type:<nerd-font|unicode>;size:<large|small>`, например `--list-style 'type:unicode;size:small'`.
  - `size` меняет только маркеры Nerd Font. Для Unicode принимаются оба значения, но набор остаётся одинаковым; его отображение зависит от шрифта. Проверено на семействах Nerd Font, особенно JetBrainsMono Nerd Font.
- `-D, --definition-marker-style <unicode|nerd-font>` - выводит описания definition list со встроенным Unicode- или Nerd Font-маркером.
- `--uniform-list-marker <level:1-4|icon:глиф>` - использует один маркер на всех уровнях вложенности (только с `--list-style`). `level:2` берёт второй маркер выбранного набора, а `icon:*` задаёт собственный глиф.
- `--custom-list <уровень>:<иконка>[:<цвет>];...` - переопределяет иконку и/или цвет маркера для конкретных уровней вложенности (только с `--list-style`). Уровень - 1-based; иконка - глиф маркера. Цвета принимают именованные (`red`), hex (`#ff0000`), rgb (`255,0,0`) и `ansi(N)`.
  - Иконка + цвет:  `--custom-list '1:*:yellow'`   маркер `*` жёлтого цвета
  - Только иконка:  `--custom-list '1:>'`          маркер `>` в цвете темы
  - Только цвет:    `--custom-list '1:red'`        встроенная иконка, красный цвет

<div data-mdv-preview="list-style"></div>

<div data-mdv-preview="uniform-list-marker"></div>

<div data-mdv-preview="custom-list"></div>

<div data-mdv-preview="definition-marker-style"></div>

## Структурированные переопределения

```yaml
checkbox_style: square
custom_checkbox:
  " ":
    icon: "!"
    color: yellow
  x:
    color: green
list_style: "type:unicode;size:small"
uniform_list_marker: "level:2"
custom_list:
  1:
    icon: "*"
    color: yellow
  2:
    color: red
definition_marker_style: unicode
```

Уровни списков начинаются с единицы.

- Наряду с позиционными формами CLI принимает именованные, например `1:icon=*,color=yellow`.
- Записи и свойства объединяются со значениями конфигурации и пресета меньшего приоритета; явный `null` очищает свойство или запись, корневые `null` и `{}` очищают всю настройку.
- Без `definition_marker_style` определения сохраняют отступ в два пробела.
- Нумерованные списки сохраняют номера, настройка маркеров относится к ненумерованным.
