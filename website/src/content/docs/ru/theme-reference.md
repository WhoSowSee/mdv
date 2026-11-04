---
title: "Поля темы"
description: "Все пути цветов темы, правила null, строчное оформление и цвета просмотрщика."
slug: "theme-reference"
group: "Customization"
order: 31
---

## Поля темы

В файле темы обязателен непустой `name`. `description` необязателен, а `extends` называет встроенную тему или пользовательскую тему, загруженную раньше по алфавиту. Пропущенные поля наследуются от основы; без `extends` основой служит `terminal`.

| Раздел | Поля | Назначение |
| --- | --- | --- |
| Корень | `text`, `text_light`, `h1`...`h6`, `quote`, `link`, `error`, `warning` | Текст, приглушённые подписи, заголовки, цитаты, ссылки и сообщения. |
| Корень | `background`, `details_border`, `horizontal_rule`, `footnote_separator` | Необязательный фон и независимые цвета структурных элементов. |
| `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough`, `highlight` | `text`, `background` | Цвета строчного текста и его фона. |
| `line_number` | `number`, `separator` | Номера строк и разделитель. |
| `table` | `header`, `border` | Заголовки и линии таблиц, включая разделители блоков. |
| `math` | `text`, `border` | Текст и рамки формул. |
| `front_matter` | `title`, `key`, `value`, `border` | Заголовок панели свойств, ключи, значения и рамка. |
| `code_block` | `label`, `border` | Общий цвет имени/значка и рамка кода. |
| `callout` | `label`, `border`, `palette` | Подписи/значки выносок, рамки и цвета типов. |
| `callout.palette` | `note`, `abstract`, `info`, `todo`, `tip`, `success`, `question`, `warning`, `failure`, `danger`, `bug`, `example`, `quote` | Отдельный цвет каждого типа выноски. |
| `list` | `ordered`, `unordered` | Нумерованные и ненумерованные маркеры. |
| `todo` | `checked`, `unchecked` | Цвета состояний задач. |
| `syntax` | `keyword`, `string`, `comment`, `number`, `operator`, `function`, `variable`, `type_name` | Роли подсветки синтаксиса. |
| `inline_style` | `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough`, `highlight` | Атрибуты оформления элементов, описаны ниже. |
| `pager` | `transparent`, `title`, `help`, `file_name`, `progress`, `matches`, `selection`, `search`, `search_current` | Палитра встроенного просмотрщика, описана ниже. |

## Форматы цветов и пути CLI

Цвета принимают имена вроде `red` и `darkgrey`, hex `#rrggbb` или `#rgb`, RGB `r,g,b`, ANSI `ansi(N)` или индекс от 0 до 255, а также `reset` для стандартного цвета терминала.

- Необязательные переопределения также принимают `null` и `none`.
- Hex-значения в YAML заключаются в кавычки.
- Присваивания CLI разделяются `;`, вложенные пути используют `:`.
- `custom_code_theme` принимает обычные ключи синтаксиса и пути `syntax:`.

<div data-mdv-preview="custom-theme"></div>

<div data-mdv-preview="custom-code-theme"></div>

```yaml
custom_theme:
  callout:
    palette:
      note: blue
custom_code_theme:
  keyword: "#89b4fa"
```

## Наследование и null

Пропущенные значения наследуются по полям.

- В файлах тем null для необязательных рамок, фонов, `callout.label` и цветов строки состояния очищает унаследованные переопределения.
- Null для `strong_emphasis.text`, `highlight.text`, обязательных цветов текста и `highlight.background` сохраняет базовые значения.
- В переопределениях цветов необязательные цвета можно очистить, но обязательные цвета текста и `highlight.background` не принимают null.
- Очистка цвета рамки сохраняет её символы в стандартном цвете текста.
- `reset` явно включает стандартный цвет терминала, а не удаляет присваивание.

Null для цветов выделения и поиска просмотрщика возвращает автоматическое/стандартное выделение.

- Markdown `highlight.background` окрашивает `==text==` независимо от поиска и выделения просмотрщика.
- Порядок источников: выбранная тема, конфигурация, пресет, CLI.
- Внутри каждого источника порядок цветов синтаксиса: `custom_theme`, `code_theme`, `custom_code_theme`; частичные присваивания большего приоритета сохраняют остальные поля.

## Строчное оформление

Каждый элемент `inline_style` принимает `backticks`, `bold`, `italic`, `underline`, `strikethrough`.

- По умолчанию emphasis использует курсив, strong жирное начертание, strong_emphasis оба, code обратные кавычки, strikethrough зачёркивание, highlight только свои цвета.
- Пропущенные и null-атрибуты сохраняют настройки, явный false отключает атрибут.
- Переопределения объединяются по элементам и свойствам.

<div data-mdv-preview="inline-style"></div>

## Палитра просмотрщика

`pager.transparent: true` убирает фоны строки состояния и справки.

- `title`, `help`, `file_name`, `progress`, `matches` окрашивают соответствующие элементы интерфейса.
- `selection`, `search`, `search_current` принимают по два поля: `text` и `background`.
- Null для полей выделения восстанавливает автоматическое/стандартное выделение.
- `--color-depth` и `--color never` действуют и на эту палитру.

```yaml
pager:
  transparent: true
  title: blue
  search_current:
    text: black
    background: yellow
```

Старые корневые `border`, `details`, плоские имена вроде `code_background` и `math_border`, а также `pager_status_bar_transparent` не принимаются. Используйте поля, перечисленные выше.
