---
title: "Полный справочник CLI"
description: "Все параметры командной строки, их форматы, допустимые значения и поведение."
slug: "cli"
group: "Reference"
order: 43
---

## Вывод и рабочий процесс

<section class="cli-option" aria-label="--html">

### `--html`

Печать HTML вместо терминального форматирования.

</section>

<section class="cli-option" aria-label="--render-html">

### `--render-html`

<p class="cli-syntax"><code>-E, --render-html</code></p>

Рендер встроенных HTML-фрагментов как форматированного терминального содержимого вместо буквального показа тегов.

- Вложенность HTML ограничена 64 DOM-элементами.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="render-html"></div>

</details>

</section>

<section class="cli-option" aria-label="--line-numbers">

### `--line-numbers`

<p class="cli-syntax"><code>-N, --line-numbers [&lt;MODE&gt;]</code></p>

Добавление номеров строк в терминальный вывод и pager.

- Без режима нумеруются отрендеренные строки
- `source` использует физические строки Markdown
- `separator` добавляет разделитель
- `"source;separator"` сочетает оба режима.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="line-numbers"></div>

</details>

</section>

<section class="cli-option" aria-label="--color">

### `--color`

<p class="cli-syntax"><code>--color &lt;auto|always|never&gt;</code></p>

Управление терминальным оформлением.

- `auto` включает ANSI и OSC 8 только при выводе в терминал
- `always` сохраняет оформление в pipe
- `never` выводит отрендерированный текст без оформления.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="color"></div>

</details>

</section>

<section class="cli-option" aria-label="--color-depth">

### `--color-depth`

<p class="cli-syntax"><code>--color-depth &lt;auto|16|256|truecolor&gt;</code></p>

Автоматическое определение палитры терминала или её явное ограничение.

- Действует на цвета документа и интерактивного интерфейса; по умолчанию `auto`.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="color-depth"></div>

</details>

</section>

<section class="cli-option" aria-label="--hide-comments">

### `--hide-comments`

Скрытие Markdown-комментариев в итоговом выводе.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="hide-comments"></div>

</details>

</section>

<section class="cli-option" aria-label="--front-matter">

### `--front-matter`

<p class="cli-syntax"><code>--front-matter &lt;hidden|panel|table|plain|inline|blocks|code|source&gt;</code></p>

Управление YAML-словарём свойств с точными разделителями `---`, начинающимся с первой строки.

- `hidden` скрывает его
- `panel` показывает панель свойств
- `table` выводит колонки «Property» и «Value»
- `plain` - нормализованные строки «ключ: значение»
- `inline` объединяет свойства в одну переносимую строку
- `blocks` помещает значение под ключом
- `code` выводит YAML с подсветкой
- `source` разбирает весь ввод как обычный Markdown.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="front-matter"></div>

</details>

</section>

<section class="cli-option" aria-label="--theme-info">

### `--theme-info`

<p class="cli-syntax"><code>--theme-info [FILE]</code></p>

Показ активной палитры; при указании `FILE` рендерит документ вместе со сведениями о теме.

</section>

<section class="cli-option" aria-label="--from">

### `--from`

<p class="cli-syntax"><code>--from &lt;TEXT&gt;</code></p>

Рендер начиная с первой строки, содержащей `<TEXT>`.

- Добавление `:<lines>` ограничит число строк (например
- `--from "Install:20"`).

</section>

<section class="cli-option" aria-label="--reverse">

### `--reverse`

<p class="cli-syntax"><code>-r, --reverse</code></p>

Рендер документа с конца, сохраняя форматирование блоков.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="reverse"></div>

</details>

</section>

<section class="cli-option" aria-label="--pager">

### `--pager`

<p class="cli-syntax"><code>-p, --pager[=&lt;COMMAND&gt;]</code></p>

Открыть результат в pager.

- Голый флаг использует `MDV_PAGER`, если переменная задана, иначе pager по умолчанию.
- `--pager=default` выбирает backend по умолчанию (сейчас встроенный `minus`)
- `--pager=builtin` явно выбирает `minus`, а команда вроде `--pager="less -R"` получает отрендерированный вывод через stdin

<details class="mdv-reference-preview">
<summary>Превью встроенного просмотрщика</summary>
<div data-mdv-preview="builtin-pager"></div>
</details>

</section>

<section class="cli-option" aria-label="--interactive">

### `--interactive`

<p class="cli-syntax"><code>-i, --interactive</code></p>

Открыть интерактивный браузер документов.

- Каталог открывает выбор документов, а файл или стандартный ввод сразу открывается в выбранном просмотрщике.
- Запуск `mdv` без аргументов в терминале открывает текущий каталог, а путь к каталогу открывает указанный каталог.
- При перенаправленном выводе браузер автоматически не запускается.
- Браузер рекурсивно находит Markdown-файлы с учётом скрытых путей и `.gitignore`.

<details class="mdv-reference-preview">
<summary>Превью выбора документов</summary>
<div data-mdv-preview="document-browser"></div>
</details>

</section>

<section class="cli-option" aria-label="--monitor">

### `--monitor`

Наблюдение за файлом и автоматический перерендер при изменениях.

</section>

<section class="cli-option" aria-label="--config-file">

### `--config-file`

<p class="cli-syntax"><code>-F, --config-file &lt;CONFIG_DIR&gt;</code></p>

Чтение настроек из указанного каталога.

</section>

<section class="cli-option" aria-label="--preset">

### `--preset`

<p class="cli-syntax"><code>-P, --preset &lt;NAME&gt;</code></p>

Применение встроенного или пользовательского пресета поверх конфигурационного файла и до явных параметров CLI.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="preset"></div>

</details>

</section>

<section class="cli-option" aria-label="--preset-info">

### `--preset-info`

Вывод списка доступных пресетов, если позиционный `FILE` не указан.

- Команда `mdv --preset-info FILE --preset NAME` рендерит документ и печатает `Current preset: NAME`; с `FILE`, но без `--preset`, флаг не меняет рендеринг.

</section>

<section class="cli-option" aria-label="--no-config">

### `--no-config`

<p class="cli-syntax"><code>-n, --no-config</code></p>

Игнорирование `config.yaml`/`config.yml`; пресеты остаются доступными.

</section>

<section class="cli-option" aria-label="--init-config">

### `--init-config`

<p class="cli-syntax"><code>--init-config [CONFIG_DIR]</code></p>

Создание конфигурационного файла по умолчанию.

- Используется указанный каталог
- `--config-file`
- `MDV_CONFIG_PATH` или каталог по умолчанию.

</section>

## Темы

<section class="cli-option" aria-label="--theme">

### `--theme`

<p class="cli-syntax"><code>-t, --theme &lt;NAME&gt;</code></p>

Выбор встроенной или пользовательской темы (по умолчанию `terminal`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-theme">

### `--code-theme`

<p class="cli-syntax"><code>-T, --code-theme &lt;NAME&gt;</code></p>

Отдельная палитра подсветки кода.

- Без явного выбора код использует палитру темы документа.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--syntaxes-dir">

### `--syntaxes-dir`

<p class="cli-syntax"><code>--syntaxes-dir &lt;DIR&gt;</code></p>

Рекурсивная загрузка пользовательских `.sublime-syntax` поверх встроенного набора.

- Пользовательские определения имеют приоритет при совпадении со встроенными.

</section>

<section class="cli-option" aria-label="--code-block-style">

### `--code-block-style`

<p class="cli-syntax"><code>-b, --code-block-style &lt;basic|simple|pretty&gt;[:show-name;show-icon]</code></p>

Стиль оформления блоков кода: безрамочный блок с отступом, одинарная граница или полная рамка.

- По умолчанию подпись скрыта
- `show-name` показывает название языка
- `show-icon` - его иконку, обе опции можно комбинировать (по умолчанию `basic`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-block-style"></div>

<div data-mdv-preview="code-labels"></div>

</details>

</section>

<section class="cli-option" aria-label="--math-block-style">

### `--math-block-style`

<p class="cli-syntax"><code>--math-block-style &lt;basic|simple|pretty&gt;</code></p>

Стиль display-формул и fenced-блоков `math`/`latex`/`tex`.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="math-block-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-line-numbers">

### `--code-line-numbers`

<p class="cli-syntax"><code>-K, --code-line-numbers [&lt;MODE&gt;]</code></p>

Нумерация строк внутри каждого блока кода.

- Без режима нумеруется каждая перенесённая терминальная строка
- `source` нумерует физические строки кода и оставляет продолжения без номера
- `separator` добавляет разделитель
- `"source;separator"` сочетает оба режима.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-line-numbers"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-theme">

### `--custom-theme`

<p class="cli-syntax"><code>--custom-theme &lt;key=value;...&gt;</code></p>

Переопределение цветов интерфейса поверх выбранной темы.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--inline-style">

### `--inline-style`

<p class="cli-syntax"><code>--inline-style &lt;STYLES&gt;</code></p>

Переопределение оформления `emphasis`

- `strong`
- `strong_emphasis`
- `code`
- `strikethrough` и `highlight`.
- Записи формата `element:property=true,property=false` разделяются `;`; доступны свойства `backticks`
- `bold`
- `italic`
- `underline` и `strikethrough` (например
- `--inline-style 'code:backticks=false,bold=true;highlight:underline=true'`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="inline-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-code-theme">

### `--custom-code-theme`

<p class="cli-syntax"><code>--custom-code-theme &lt;key=value;...&gt;</code></p>

Переопределение цветов подсветки кода в том же формате, что и `--custom-theme`.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-code-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-code-block">

### `--custom-code-block`

<p class="cli-syntax"><code>--custom-code-block &lt;lang:icon=...,label=...,aliases=...|...&gt;[;...]</code></p>

Переопределение иконки, подписи и псевдонимов для конкретных языков блоков кода.

- Несколько языков разделяются `;`, опции внутри одного языка - `,`, а псевдонимы - `|` (например
- `python:icon=*,label=Python,aliases=py|py3;rust:icon=R`).
- Работает для любого указанного языка, даже если его нет во встроенной карте иконок.
- Подсветка синтаксиса применяется только к языкам, которые поддерживает текущая логика подсветки mdv.
- Используйте `default:icon=...`, чтобы задать fallback-иконку для неизвестных языков.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-code-block"></div>

</details>

</section>

## Callout-блоки

<section class="cli-option" aria-label="--callout-style">

### `--callout-style`

<p class="cli-syntax"><code>-C, --callout-style &lt;pretty|simple&gt;[:show-icons;show-simple-icons;fold-icons;label-inside;uppercase]</code></p>

Стиль callout-блоков и правила отображения подписи.

- `show-icons` использует глифы Nerd Font
- `show-simple-icons` - переносимые ASCII-маркеры вроде `[i]`
- `[*]`
- `[!]` и `[x]`.
- Эти два параметра нельзя комбинировать.
- `label-inside` поддерживается только с `pretty`
- `fold-icons` требует `show-icons`.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="callout-style"></div>

<div data-mdv-preview="callout-icons"></div>

<div data-mdv-preview="callout-label"></div>

<div data-mdv-preview="callout-uppercase"></div>

<div data-mdv-preview="callout-fold"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-callout">

### `--custom-callout`

<p class="cli-syntax"><code>--custom-callout &lt;name:icon=...,color=...;...&gt;</code></p>

Переопределение или добавление подписей callout.

- `icon` и `color` можно указывать по отдельности; формат цвета как в `--custom-theme`.

- Для корректного отображения иконок, включённых через `show-icons` и `fold-icons`, нужны шрифты Nerd Fonts в терминале. Для `show-simple-icons` они не требуются.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-callout"></div>

</details>

</section>

## Чекбоксы

<section class="cli-option" aria-label="--checkbox-style">

### `--checkbox-style`

<p class="cli-syntax"><code>-x, --checkbox-style &lt;square|circle&gt;</code></p>

Рендерит чекбоксы task-list как иконки Nerd Font вместо стандартных `[ ]` / `[x]`.

- Требует шрифт Nerd Font в терминале.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="checkbox-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-checkbox">

### `--custom-checkbox`

<p class="cli-syntax"><code>--custom-checkbox &lt;PAIRS&gt;</code></p>

Переопределяет встроенные иконки чекбоксов или добавляет новые состояния (только с `--checkbox-style`).

- Каждая разделённая точкой с запятой пара задаётся как `<char>:<icon>[:<color>]` или `<char>:<color>`.
- Форма только с цветом сохраняет существующую иконку, а для нового состояния использует стандартную иконку неотмеченного чекбокса.
- Цвета принимают именованные, hex, rgb и `ansi(N)`.

- Переопределение:  `--custom-checkbox ' :!'`         заменить иконку неотмеченного
- Добавление:       `--custom-checkbox '*:*'`         добавить состояние `[*]`
- Цвет:             `--custom-checkbox ' :!:yellow'`  или `#ff0000`, `128,1,1`, `ansi(200)`
- Только цвет: `--custom-checkbox '?:red'` сохраняет значок Nerd Font для состояния `?` и задаёт красный цвет.
- Новое состояние, только цвет: `--custom-checkbox '*:yellow'` использует стандартный значок неотмеченного чекбокса и жёлтый цвет.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-checkbox"></div>

</details>

</section>

## Списки

<section class="cli-option" aria-label="--list-style">

### `--list-style`

<p class="cli-syntax"><code>-L, --list-style &lt;стиль&gt;</code></p>

Заменяет стандартные маркеры `-` неупорядоченных списков на набор иконок по уровням.

- Значение обязательно и задаётся в формате `type:<nerd-font|unicode>;size:<large|small>`, например `--list-style 'type:unicode;size:small'`.

- `size` меняет только маркеры Nerd Font. Для Unicode принимаются оба значения, но набор остаётся одинаковым; его отображение зависит от шрифта. Проверено на семействах Nerd Font, особенно JetBrainsMono Nerd Font.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="list-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--definition-marker-style">

### `--definition-marker-style`

<p class="cli-syntax"><code>-D, --definition-marker-style &lt;unicode|nerd-font&gt;</code></p>

Выводит описания definition list со встроенным Unicode- или Nerd Font-маркером.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="definition-marker-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--uniform-list-marker">

### `--uniform-list-marker`

<p class="cli-syntax"><code>--uniform-list-marker &lt;level:1-4|icon:глиф&gt;</code></p>

Использует один маркер на всех уровнях вложенности (только с `--list-style`).

- `level:2` берёт второй маркер выбранного набора
- `icon:*` задаёт собственный глиф.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="uniform-list-marker"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-list">

### `--custom-list`

<p class="cli-syntax"><code>--custom-list &lt;уровень&gt;:&lt;иконка&gt;[:&lt;цвет&gt;];...</code></p>

Переопределяет иконку и/или цвет маркера для конкретных уровней вложенности (только с `--list-style`).

- Уровень - 1-based; иконка - глиф маркера.
- Цвета принимают именованные (`red`), hex (`#ff0000`), rgb (`255,0,0`) и `ansi(N)`.

- Иконка + цвет:  `--custom-list '1:*:yellow'`   маркер `*` жёлтого цвета
- Только иконка:  `--custom-list '1:>'`          маркер `>` в цвете темы
- Только цвет:    `--custom-list '1:red'`        встроенная иконка, красный цвет

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-list"></div>

</details>

</section>

## Разметка и переносы

<section class="cli-option" aria-label="--cols">

### `--cols`

<p class="cli-syntax"><code>-c, --cols &lt;N&gt;</code></p>

Фиксированная ширина вывода.

- Без него используется определённая ширина терминала, если она не меньше 20 столбцов, затем `cols` из конфигурации, затем 80 столбцов.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="cols"></div>

</details>

</section>

<section class="cli-option" aria-label="--margin">

### `--margin`

<p class="cli-syntax"><code>-m, --margin &lt;right:N;left:N&gt;</code></p>

Задаёт левый и/или правый отступ.

- Неуказанная сторона равна `0`.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="margin"></div>

</details>

</section>

<section class="cli-option" aria-label="--tab-length">

### `--tab-length`

<p class="cli-syntax"><code>--tab-length &lt;N&gt;</code></p>

Замена символов табуляции на `N` пробелов (по умолчанию `4`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="tab-length"></div>

</details>

</section>

<section class="cli-option" aria-label="--wrap">

### `--wrap`

<p class="cli-syntax"><code>-w, --wrap &lt;char|word|none&gt;</code></p>

Стратегия переноса текста, кода, callout-блоков и содержимого ячеек таблиц (по умолчанию `char`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="wrap"></div>

</details>

</section>

<section class="cli-option" aria-label="--reflow">

### `--reflow`

Склеивает внутриабзацные мягкие переносы строк (soft breaks) и заново заполняет каждую строку до ширины переноса, так что жёстко разбитый исходник перетекает под узкие экраны.

- Требует включённого переноса; жёсткие переносы сохраняются.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="reflow"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-wrap">

### `--table-wrap`

<p class="cli-syntax"><code>-W, --table-wrap &lt;fit|wrap|none&gt;</code></p>

Геометрия таблиц: подогнать ячейки, разделить колонки на блоки или разрешить переполнение (по умолчанию `fit`).

- Точки переноса в ячейках определяет `--wrap`.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="table-wrap"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-borders">

### `--table-borders`

<p class="cli-syntax"><code>-B, --table-borders</code></p>

Возвращает полные скруглённые рамки у таблиц.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="table-borders"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-smart-indent">

### `--table-smart-indent`

<p class="cli-syntax"><code>-S, --table-smart-indent</code></p>

Автоматический подбор отступа таблицы в зависимости от доступной ширины.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="table-smart-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--heading-layout">

### `--heading-layout`

<p class="cli-syntax"><code>-H, --heading-layout &lt;level|center|flat|none&gt;</code></p>

Схема выравнивания заголовков (по умолчанию `level`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="heading-layout"></div>

</details>

</section>

<section class="cli-option" aria-label="--show-heading-markers">

### `--show-heading-markers`

Добавляет перед заголовками markdown-маркеры, соответствующие их уровню.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="show-heading-markers"></div>

</details>

</section>

<section class="cli-option" aria-label="--smart-indent">

### `--smart-indent`

<p class="cli-syntax"><code>-I, --smart-indent</code></p>

Сглаживание скачков отступов между уровнями заголовков в режиме `level`.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="smart-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-wrap-indent">

### `--code-wrap-indent`

<p class="cli-syntax"><code>--code-wrap-indent &lt;none|base|double&gt;</code></p>

Управляет «висячим» отступом при переносе строк внутри блоков кода (по умолчанию `double`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-wrap-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--block-spacing">

### `--block-spacing`

<p class="cli-syntax"><code>--block-spacing &lt;спецификация&gt;</code></p>

Переопределяет количество пустых строк сверху и снизу для отдельных блоков.

- Неуказанные стороны сохраняют стандартные значения, а отступы соседних блоков схлопываются до большего значения.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="block-spacing"></div>

</details>

</section>

<section class="cli-option" aria-label="--horizontal-rule-style">

### `--horizontal-rule-style`

<p class="cli-syntax"><code>--horizontal-rule-style &lt;pretty|simple&gt;</code></p>

Рисует разделители Markdown и HTML `<hr>` с декоративными символами по краям или сплошной линией.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="horizontal-rule-style"></div>

</details>

</section>

## Видимость элементов

<section class="cli-option" aria-label="--show-empty-elements">

### `--show-empty-elements`

Отображение обычно скрытых пустых заголовков, списков, цитат, блоков кода и таблиц.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="show-empty-elements"></div>

</details>

</section>

<section class="cli-option" aria-label="--no-code-guessing">

### `--no-code-guessing`

Отключение эвристики определения языка; неизвестные блоки остаются текстом.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-guessing"></div>

</details>

</section>

## Ссылки

<section class="cli-option" aria-label="--link-style">

### `--link-style`

<p class="cli-syntax"><code>-u, --link-style &lt;clickable|fclickable|inline|inlinetable|endtable|hide&gt;</code></p>

Способ отображения ссылок (по умолчанию `clickable`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="link-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--link-overflow">

### `--link-overflow`

<p class="cli-syntax"><code>-l, --link-overflow &lt;wrap|cut|tablecut|none&gt;</code></p>

Определяет, переносить, обрезать или разрешать переполнение длинных ссылок (по умолчанию `wrap`).

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="link-overflow"></div>

</details>

</section>

## Сноски

<section class="cli-option" aria-label="--footnote-style">

### `--footnote-style`

<p class="cli-syntax"><code>--footnote-style &lt;endnotes|attached&gt;</code></p>

Размещение сносок в конце документа или сразу после блока со ссылкой, включая абзацы, таблицы и элементы списков.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="footnote-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--missing-footnote-style">

### `--missing-footnote-style`

<p class="cli-syntax"><code>--missing-footnote-style &lt;show|hide&gt;</code></p>

Поведение для отсутствующих, некорректных или пустых определений сносок.

- `show` выводит сообщение-заглушку в блоке сносок `hide` полностью скрывает такие записи.

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="missing-footnote-style"></div>

</details>

</section>

## Справочная информация

<section class="cli-option" aria-label="mdv help">

### `mdv help`

Открывает полный вывод `mdv --help` в выбранном pager.

- Если задан `MDV_PAGER`, используется он, иначе встроенный pager; при перенаправлении ввода или вывода та же справка печатается напрямую.

</section>

<section class="cli-option" aria-label="-h">

### `-h`

Краткая сводка параметров.

</section>

<section class="cli-option" aria-label="--help">

### `--help`

Полная справка с описаниями значений, форматами и примерами.

</section>

<section class="cli-option" aria-label="--version">

### `--version`

<p class="cli-syntax"><code>-V, --version</code></p>

Отображение текущей версии.

</section>
