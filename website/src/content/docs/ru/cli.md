---
title: "Полный справочник CLI"
description: "Все параметры командной строки, их форматы, допустимые значения и поведение."
slug: "cli"
group: "Reference"
order: 43
---

Ниже описаны встроенные настройки. Как их меняют YAML, пресеты, переменные окружения и параметры CLI, объясняет [руководство по конфигурации](../configuration/).


## Вывод и рабочий процесс

<section class="cli-option" aria-label="--html">

### `--html`

Экспортирует отрендеренный документ в HTML.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Терминальный вывод</strong><br> Применяет терминальную разметку и выбранный режим цвета.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --html example.md > example.html
```

</section>

<section class="cli-option" aria-label="--render-html">

### `--render-html`

<p class="cli-syntax"><code>-E, --render-html</code></p>

Рендерит встроенные HTML-фрагменты как форматированное содержимое терминала.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> HTML-теги выводятся буквально.

Вложенность HTML ограничена 64 DOM-элементами.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --render-html example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="render-html"></div>

</details>

</section>

<section class="cli-option" aria-label="--line-numbers">

### `--line-numbers`

<p class="cli-syntax"><code>-N, --line-numbers [&lt;MODE&gt;]</code></p>

Добавляет номера строк в терминальный вывод и просмотрщик.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Строки документа выводятся без поля для номеров.

- Если передать флаг без значения, нумеруются строки вывода без разделителя.
- `source` нумерует физические строки исходного Markdown.
- `separator` нумерует строки вывода и добавляет разделитель.
- `source;separator` нумерует исходные строки и добавляет разделитель.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --line-numbers 'source;separator' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="line-numbers"></div>

</details>

</section>

<section class="cli-option" aria-label="--color">

### `--color`

<p class="cli-syntax"><code>--color &lt;WHEN&gt;</code></p>

Управляет ANSI-оформлением и гиперссылками OSC 8.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`auto`</strong><br> Включает терминальное оформление и гиперссылки OSC 8 при выводе в терминал.

- `auto` включает оформление и гиперссылки при выводе в терминал.
- `always` сохраняет оформление и гиперссылки и при перенаправлении stdout.
- `never` выводит отрендерированный текст без оформления.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --color always example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="color"></div>

</details>

</section>

<section class="cli-option" aria-label="--color-depth">

### `--color-depth`

<p class="cli-syntax"><code>--color-depth &lt;DEPTH&gt;</code></p>

Задаёт предел палитры документа и интерактивного интерфейса. Действует при включённом терминальном оформлении.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`auto`</strong><br> Определяет палитру, которую поддерживает терминал.

- `auto` определяет возможности терминала.
- `16` использует стандартную палитру ANSI.
- `256` использует палитру ANSI из 256 цветов.
- `truecolor` использует 24-битные цвета RGB.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --color-depth 256 example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="color-depth"></div>

</details>

</section>

<section class="cli-option" aria-label="--hide-comments">

### `--hide-comments`

Скрывает Markdown-комментарии в итоговом выводе.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Комментарии видны.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --hide-comments example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="hide-comments"></div>

</details>

</section>

<section class="cli-option" aria-label="--front-matter">

### `--front-matter`

<p class="cli-syntax"><code>--front-matter &lt;MODE&gt;</code></p>

Управляет выводом YAML-словаря между строками с точным разделителем `---`. Открывающий разделитель должен быть на первой строке.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`hidden`</strong><br> Скрывает начальный блок YAML в отрендерированном выводе.

- `hidden` скрывает front matter.
- `panel` показывает панель свойств.
- `table` показывает столбцы `Property` и `Value`.
- `plain` выводит нормализованные строки «ключ: значение».
- `inline` объединяет свойства в один абзац с переносами.
- `blocks` помещает каждое значение под его ключом.
- `code` выводит YAML с подсветкой.
- `source` разбирает весь ввод как обычный Markdown.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --front-matter panel example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="front-matter"></div>

</details>

</section>

<section class="cli-option" aria-label="--theme-info">

### `--theme-info`

<p class="cli-syntax"><code>--theme-info [FILE]</code></p>

Без значения показывает активные и доступные темы. Если указать файл, выводит документ вместе со сведениями об активных темах.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Сведения о темах скрыты.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --theme-info example.md
```

</section>

<section class="cli-option" aria-label="--from">

### `--from`

<p class="cli-syntax"><code>--from &lt;TEXT&gt;[:&lt;LINES&gt;]</code></p>

Начинает вывод с первой строки, содержащей `<TEXT>`. Суффикс `:<LINES>` ограничивает число строк исходника с учётом найденной строки. Если совпадений нет, вывод начинается с первой строки.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Весь документ</strong><br> Выводится весь документ без фильтрации.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --from 'Installation:20' README.md
```

</section>

<section class="cli-option" aria-label="--reverse">

### `--reverse`

<p class="cli-syntax"><code>-r, --reverse</code></p>

Выводит документ с конца, сохраняя форматирование блоков.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Исходный порядок</strong><br> Выводит документ с начала.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --reverse example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="reverse"></div>

</details>

</section>

<section class="cli-option" aria-label="--pager">

### `--pager`

<p class="cli-syntax"><code>-p, --pager[=&lt;COMMAND&gt;]</code></p>

Открывает результат в постраничном просмотрщике, если stdout подключён к терминалу. При перенаправлении выводит результат напрямую.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Прямой вывод</strong><br> Обычный вывод документа идёт напрямую в stdout.

- `--pager` использует `MDV_PAGER`, если переменная задана, иначе встроенный просмотрщик `minus`.
- `--pager=default` и `--pager=builtin` выбирают встроенный `minus`.
- `--pager=<COMMAND>` запускает внешний просмотрщик и передаёт результат через stdin.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --pager=builtin example.md
```

<details class="mdv-reference-preview">
<summary>Превью встроенного просмотрщика</summary>
<div data-mdv-preview="builtin-pager"></div>
</details>

</section>

<section class="cli-option" aria-label="--interactive">

### `--interactive`

<p class="cli-syntax"><code>-i, --interactive</code></p>

Открывает браузер документов для каталога или сразу показывает файл либо стандартный ввод в выбранном просмотрщике. Требует терминал на stdout и несовместим с `--pager`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Автоматически</strong><br> Путь к каталогу открывает браузер. Без аргументов при терминальных stdin и stdout открывается текущий каталог. Файлы и перенаправленный ввод выводятся напрямую.

Браузер рекурсивно ищет Markdown-файлы с учётом скрытых путей и правил `.gitignore`.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --interactive .
```

<details class="mdv-reference-preview">
<summary>Превью выбора документов</summary>
<div data-mdv-preview="document-browser"></div>
</details>

</section>

<section class="cli-option" aria-label="--monitor">

### `--monitor`

Отслеживает изменения файла и обновляет вывод. Действует при прямом выводе из файла.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Документ выводится один раз, изменения файла не отслеживаются.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --monitor example.md
```

</section>

<section class="cli-option" aria-label="--config-file">

### `--config-file`

<p class="cli-syntax"><code>-F, --config-file &lt;CONFIG_DIR&gt;</code></p>

Ищет `config.yaml`, затем `config.yml` в указанном каталоге. Если их нет, проверяет `MDV_CONFIG_PATH`, когда переменная задана.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Автопоиск</strong><br> Поиск идёт в `MDV_CONFIG_PATH`, если переменная задана, иначе в [стандартном каталоге конфигурации](../configuration/).

<strong class="cli-example-label">Пример</strong>

```sh
mdv --config-file ./config example.md
```

</section>

<section class="cli-option" aria-label="--preset">

### `--preset`

<p class="cli-syntax"><code>-P, --preset &lt;NAME&gt;</code></p>

Применяет встроенный или пользовательский пресет после загрузки конфигурации и перед обработкой переменных окружения и параметров CLI.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Без пресета</strong><br> Настройки из конфигурации дополняются переменными окружения и параметрами CLI.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --preset reader example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="preset"></div>

</details>

</section>

<section class="cli-option" aria-label="--preset-info">

### `--preset-info`

Выводит список доступных пресетов, если позиционный файл не указан. Если передать файл и `--preset`, выводит документ и строку `Current preset: NAME`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Сведения о пресетах скрыты.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --preset-info example.md --preset pretty
```

</section>

<section class="cli-option" aria-label="--no-config">

### `--no-config`

<p class="cli-syntax"><code>-n, --no-config</code></p>

Пропускает загрузку `config.yaml` и `config.yml`. Пресеты остаются доступны.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Загрузка включена</strong><br> Загружается первый найденный файл конфигурации.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --no-config example.md
```

</section>

<section class="cli-option" aria-label="--init-config">

### `--init-config`

<p class="cli-syntax"><code>--init-config [CONFIG_DIR]</code></p>

Создаёт `config.yaml` и завершает работу. Если файл уже существует, сохраняет его и возвращает ошибку.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">По запросу</strong><br> Обычный запуск оставляет конфигурационные файлы без изменений.

Если передать `--init-config` без значения, mdv берёт каталог из `--config-file`, затем из `MDV_CONFIG_PATH` или из стандартного пути.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --init-config ./config
```

</section>

## Темы

<section class="cli-option" aria-label="--theme">

### `--theme`

<p class="cli-syntax"><code>-t, --theme &lt;NAME&gt;</code></p>

Выбирает встроенную или [пользовательскую тему](../themes/).

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`terminal`</strong><br> Использует встроенную палитру терминала.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --theme catppuccin example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-theme">

### `--code-theme`

<p class="cli-syntax"><code>-T, --code-theme &lt;NAME&gt;</code></p>

Выбирает отдельную палитру подсветки синтаксиса. Принимает те же имена тем, что и `--theme`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Тема документа</strong><br> Подсветка синтаксиса использует тему документа.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --code-theme monokai example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--syntaxes-dir">

### `--syntaxes-dir`

<p class="cli-syntax"><code>--syntaxes-dir &lt;DIR&gt;</code></p>

Рекурсивно загружает файлы `.sublime-syntax`. Пользовательские определения имеют приоритет при совпадении со встроенными. Относительный путь в CLI отсчитывается от рабочего каталога.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Встроенные синтаксисы</strong><br> Используется только встроенный набор синтаксисов.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --syntaxes-dir ./syntaxes example.md
```

</section>

<section class="cli-option" aria-label="--code-block-style">

### `--code-block-style`

<p class="cli-syntax"><code>-b, --code-block-style &lt;STYLE&gt;[:&lt;OPTIONS&gt;]</code></p>

Задаёт оформление и подпись блоков кода.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`basic`</strong><br> Выводит код с отступом, без рамки и подписи языка.

**Стили**

- `basic` добавляет отступ без рамки.
- `simple` добавляет левую границу.
- `pretty` рисует рамку.

Параметр `show-name` показывает название языка, а `show-icon` добавляет его иконку. Их можно объединить через `;`. Для иконок нужен шрифт Nerd Font.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --code-block-style 'pretty:show-name;show-icon' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-block-style"></div>

<div data-mdv-preview="code-labels"></div>

</details>

</section>

<section class="cli-option" aria-label="--math-block-style">

### `--math-block-style`

<p class="cli-syntax"><code>--math-block-style &lt;STYLE&gt;</code></p>

Задаёт оформление блочных формул и ограждённых блоков `math`, `latex` и `tex`. У формул нет подписей, иконок и номеров строк.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`basic`</strong><br> Выводит формулы с отступом, без рамки.

- `basic` добавляет отступ без рамки.
- `simple` добавляет левую границу.
- `pretty` рисует рамку.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --math-block-style pretty example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="math-block-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-line-numbers">

### `--code-line-numbers`

<p class="cli-syntax"><code>-K, --code-line-numbers [&lt;MODE&gt;]</code></p>

Добавляет номера строк внутри блоков кода. Каждый блок нумеруется отдельно.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Строки кода выводятся без поля для номеров.

- Если передать флаг без значения, нумеруются строки вывода без разделителя.
- `source` нумерует физические строки кода и оставляет продолжения без номера.
- `separator` нумерует строки вывода и добавляет разделитель.
- `source;separator` нумерует исходные строки и добавляет разделитель.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --code-line-numbers 'source;separator' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-line-numbers"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-theme">

### `--custom-theme`

<p class="cli-syntax"><code>--custom-theme &lt;key=value;...&gt;</code></p>

Переопределяет цвета выбранной темы.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Без переопределений</strong><br> Используется палитра выбранной темы.

Разделяйте назначения символом `;`. Для цветов верхнего уровня используется `ключ=значение`, для вложенных цветов используется `раздел:поле=значение`. Доступные ключи перечислены в [справочнике полей темы](../theme-reference/). Цвета принимаются в именованном формате, hex, RGB или `ansi(N)`.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --custom-theme 'line_number:number=grey;h2=blue' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--inline-style">

### `--inline-style`

<p class="cli-syntax"><code>--inline-style &lt;STYLES&gt;</code></p>

Переопределяет оформление строчных элементов Markdown.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Без переопределений</strong><br> Используется оформление активной темы.

Формат записи `element:property=true,property=false`. Разделяйте элементы символом `;`, а свойства символом `,`.

**Элементы** `emphasis`, `strong`, `strong_emphasis`, `code`, `strikethrough`, `highlight`.

**Свойства** `backticks`, `bold`, `italic`, `underline`, `strikethrough`.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --inline-style 'code:backticks=false,bold=true;highlight:underline=true' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="inline-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-code-theme">

### `--custom-code-theme`

<p class="cli-syntax"><code>--custom-code-theme &lt;key=value;...&gt;</code></p>

Переопределяет цвета подсветки синтаксиса.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Без переопределений</strong><br> Используется текущая палитра подсветки без этих переопределений.

Разделяйте назначения `key=value` символом `;`. Поддерживаются обе формы ключей, например `keyword` и `syntax:keyword`.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --custom-code-theme 'keyword=#ffffff;string=128,0,128' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-code-theme"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-code-block">

### `--custom-code-block`

<p class="cli-syntax"><code>--custom-code-block &lt;BLOCKS&gt;</code></p>

Переопределяет иконки, подписи и псевдонимы языковых меток в блоках кода. Для подсветки требуется подходящее определение синтаксиса.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Без переопределений</strong><br> Используются встроенные иконки, подписи и псевдонимы.

Разделяйте языки символом `;`, свойства символом `,`, а псевдонимы символом `|`. Явный список псевдонимов заменяет унаследованный. Чтобы очистить список, используйте `aliases=[]`. Запись `default:icon=…` задаёт иконку для неизвестных языков.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --code-block-style pretty:show-name --custom-code-block 'rust:label=Rust sample' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-code-block"></div>

</details>

</section>

## Callout-блоки

<section class="cli-option" aria-label="--callout-style">

### `--callout-style`

<p class="cli-syntax"><code>-C, --callout-style &lt;STYLE&gt;[:&lt;OPTIONS&gt;]</code></p>

Задаёт оформление callout-блоков, иконок и подписей.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`pretty`</strong><br> Рисует рамку с подписью сверху, иконки выключены.

**Стили**

- `pretty` рисует рамку.
- `simple` добавляет левую границу.

**Дополнительные параметры**

- `show-icons` использует иконки Nerd Font.
- `show-simple-icons` использует ASCII-маркеры, например `[i]`, `[*]`, `[!]` и `[x]`. Несовместим с `show-icons`.
- `fold-icons` добавляет маркеры сворачивания и требует `show-icons`.
- `label-inside` переносит подпись внутрь рамки и требует `pretty`.
- `uppercase` выводит подписи прописными буквами.

Разделяйте дополнительные параметры символом `;`.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --callout-style 'pretty:show-icons;fold-icons;label-inside' example.md
```

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

Переопределяет или добавляет иконки и цвета для типов callout-блоков. Можно указать одно свойство или оба.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Без переопределений</strong><br> Используются встроенные стили callout-блоков и цвета активной темы.

Разделяйте записи символом `;`, а свойства символом `,`. Форматы цвета совпадают с [`--custom-theme`](#--custom-theme).

<strong class="cli-example-label">Пример</strong>

```sh
mdv --custom-callout 'tip:icon=*,color=red;custom:color=#ff5555' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-callout"></div>

</details>

</section>

## Чекбоксы

<section class="cli-option" aria-label="--checkbox-style">

### `--checkbox-style`

<p class="cli-syntax"><code>-x, --checkbox-style &lt;square|circle&gt;</code></p>

Выводит чекбоксы списков задач как квадратные (`square`) или круглые (`circle`) иконки. Требует шрифт Nerd Font.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Текстовые маркеры</strong><br> Используются текстовые отметки, например `[ ]` и `[x]`.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --checkbox-style square example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="checkbox-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-checkbox">

### `--custom-checkbox`

<p class="cli-syntax"><code>--custom-checkbox &lt;PAIRS&gt;</code></p>

Переопределяет иконки и цвета чекбоксов или добавляет новые состояния. Требует `--checkbox-style`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Без переопределений</strong><br> Используются иконки и цвета выбранного стиля чекбоксов и темы.

Разделяйте записи символом `;`. Форматы записи `<char>:<icon>[:<color>]`, `<char>:<color>` или `<char>:icon=<glyph>,color=<color>`. Если задан только цвет, знакомое состояние сохраняет свою иконку, а новое использует иконку неотмеченного чекбокса. Цвета принимаются в именованном формате, hex, RGB или `ansi(N)`.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --checkbox-style square --custom-checkbox ' :!:yellow' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-checkbox"></div>

</details>

</section>

## Списки

<section class="cli-option" aria-label="--list-style">

### `--list-style`

<p class="cli-syntax"><code>-L, --list-style &lt;LIST_STYLE&gt;</code></p>

Заменяет маркеры неупорядоченных списков иконками по уровням.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Текстовые маркеры</strong><br> Используется маркер `-`.

- `type` принимает `nerd-font` или `unicode`.
- `size` принимает `large` или `small` и влияет только на иконки Nerd Font.

Используйте запись `type:TYPE;size:SIZE` или укажите один из двух параметров. Для пропущенных параметров используются `type:nerd-font` и `size:large`. Иконки Nerd Font требуют соответствующий шрифт. Отображение Unicode-маркеров может зависеть от шрифта.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --list-style 'type:unicode' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="list-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--definition-marker-style">

### `--definition-marker-style`

<p class="cli-syntax"><code>-D, --definition-marker-style &lt;unicode|nerd-font&gt;</code></p>

Добавляет маркер `unicode` или `nerd-font` перед описанием определения. Для `nerd-font` нужен шрифт Nerd Font.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Только отступ</strong><br> Описание выводится с отступом в два пробела без маркера.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --definition-marker-style unicode example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="definition-marker-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--uniform-list-marker">

### `--uniform-list-marker`

<p class="cli-syntax"><code>--uniform-list-marker &lt;MARKER&gt;</code></p>

Выводит один маркер неупорядоченного списка на всех уровнях. Требует `--list-style`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">По уровню</strong><br> Используются маркеры выбранного набора для каждого уровня.

- `level:<1-4>` берёт маркер указанного уровня.
- `icon:<GLYPH>` задаёт свой маркер.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --list-style 'type:unicode' --uniform-list-marker 'level:2' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="uniform-list-marker"></div>

</details>

</section>

<section class="cli-option" aria-label="--custom-list">

### `--custom-list`

<p class="cli-syntax"><code>--custom-list &lt;PAIRS&gt;</code></p>

Переопределяет маркеры неупорядоченных списков по уровням вложенности. Требует `--list-style`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Без переопределений</strong><br> Используются маркеры выбранного набора и цвета темы.

Разделяйте записи символом `;`. Нумерация уровней начинается с 1. Форматы записи `<level>:<icon>[:<color>]`, `<level>:<color>` или `<level>:icon=<glyph>,color=<color>`. Цвета принимаются в именованном формате, hex, RGB или `ansi(N)`.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --list-style 'type:unicode' --custom-list '1:*:yellow' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="custom-list"></div>

</details>

</section>

## Разметка и переносы

<section class="cli-option" aria-label="--cols">

### `--cols`

<p class="cli-syntax"><code>-c, --cols &lt;N&gt;</code></p>

Задаёт ширину вывода в столбцах терминала.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Автоматически</strong><br> Используется определённая ширина терминала, если она не меньше 20 столбцов, затем `cols` из конфигурации, затем 80 столбцов.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --cols 72 example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="cols"></div>

</details>

</section>

<section class="cli-option" aria-label="--margin">

### `--margin`

<p class="cli-syntax"><code>-m, --margin &lt;MARGINS&gt;</code></p>

Задаёт левое и правое поля в столбцах терминала. Можно указать одну сторону или обе. Неуказанная сторона равна нулю. При экспорте в HTML поля не применяются.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">0 столбцов</strong><br> Оба поля равны нулю.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --margin 'left:4;right:5' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="margin"></div>

</details>

</section>

<section class="cli-option" aria-label="--tab-length">

### `--tab-length`

<p class="cli-syntax"><code>--tab-length &lt;N&gt;</code></p>

Задаёт число пробелов, которыми заменяется символ табуляции.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`4`</strong><br> Заменяет каждую табуляцию четырьмя пробелами.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --tab-length 2 example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="tab-length"></div>

</details>

</section>

<section class="cli-option" aria-label="--wrap">

### `--wrap`

<p class="cli-syntax"><code>-w, --wrap &lt;MODE&gt;</code></p>

Определяет, где переносить текст, код, содержимое callout-блоков и ячеек таблиц.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`char`</strong><br> Переносит по границам символов.

- `char` переносит по границам символов.
- `word` предпочитает границы слов. Слишком длинное слово может разбиваться.
- `none` отключает мягкий перенос. Текст в ячейках с ограниченной шириной всё ещё может переноситься.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --wrap word example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="wrap"></div>

</details>

</section>

<section class="cli-option" aria-label="--reflow">

### `--reflow`

Объединяет строки внутри абзаца по мягким переносам и заново разбивает текст по ширине вывода. Требует включённого переноса строк и сохраняет жёсткие переносы.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Исходные разрывы строк внутри абзацев сохраняются.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --reflow example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="reflow"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-wrap">

### `--table-wrap`

<p class="cli-syntax"><code>-W, --table-wrap &lt;MODE&gt;</code></p>

Определяет, как широкие таблицы помещаются в терминал. Если ширина ячеек ограничена, места переноса определяет `--wrap`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`fit`</strong><br> Переносит содержимое ячеек, чтобы таблица уместилась по ширине.

- `fit` переносит текст внутри ячеек, чтобы таблица поместилась по ширине.
- `wrap` разбивает широкую таблицу на блоки столбцов.
- `none` сохраняет строки ячеек без переноса и разрешает выход за границы по горизонтали.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --table-wrap wrap example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="table-wrap"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-borders">

### `--table-borders`

<p class="cli-syntax"><code>-B, --table-borders</code></p>

Рисует полную сетку таблицы со скруглёнными внешними углами.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Компактные границы</strong><br> Используются компактные границы без полной внешней рамки.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --table-borders example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="table-borders"></div>

</details>

</section>

<section class="cli-option" aria-label="--table-smart-indent">

### `--table-smart-indent`

<p class="cli-syntax"><code>-S, --table-smart-indent</code></p>

Сохраняет текущий отступ содержимого для таблицы, когда хватает места, и уменьшает его при нехватке ширины.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Автоматическая корректировка отступа выключена.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --table-smart-indent example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="table-smart-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--heading-layout">

### `--heading-layout`

<p class="cli-syntax"><code>-H, --heading-layout &lt;LAYOUT&gt;</code></p>

Управляет выравниванием заголовков и отступами содержимого под ними.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`level`</strong><br> Задаёт отступ по уровню заголовка, а его содержимое сдвигает ещё на один столбец.

- `level` сдвигает заголовки по уровню, а содержимое ещё на один столбец.
- `center` центрирует заголовки, содержимое остаётся без отступа.
- `flat` выравнивает заголовки по левому краю, содержимое сдвигается на один столбец.
- `none` выравнивает заголовки и содержимое по левому краю.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --heading-layout center example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="heading-layout"></div>

</details>

</section>

<section class="cli-option" aria-label="--show-heading-markers">

### `--show-heading-markers`

Выводит перед заголовками маркеры Markdown, соответствующие их уровню.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Маркеры заголовков скрыты.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --show-heading-markers example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="show-heading-markers"></div>

</details>

</section>

<section class="cli-option" aria-label="--smart-indent">

### `--smart-indent`

<p class="cli-syntax"><code>-I, --smart-indent</code></p>

Сглаживает скачки отступов между соседними заголовками. Действует только при `--heading-layout level`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Отступ каждого заголовка соответствует его уровню.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --heading-layout level --smart-indent example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="smart-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--code-wrap-indent">

### `--code-wrap-indent`

<p class="cli-syntax"><code>--code-wrap-indent &lt;MODE&gt;</code></p>

Задаёт висячий отступ для продолжений строк кода.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`double`</strong><br> Добавляет два пробела к исходному отступу при переносе строк кода.

- `none` отключает висячий отступ.
- `base` выравнивает продолжение по отступу исходной строки.
- `double` добавляет два пробела к отступу исходной строки.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --code-wrap-indent base example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-wrap-indent"></div>

</details>

</section>

<section class="cli-option" aria-label="--block-spacing">

### `--block-spacing`

<p class="cli-syntax"><code>--block-spacing &lt;SPACING&gt;</code></p>

Задаёт число пустых строк до и после блоков каждого типа. Между соседними блоками используется больший из двух отступов.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Встроенные отступы</strong><br> У большинства блоков по одной строке сверху и снизу. У абзацев ноль сверху и одна снизу. У `end-references` две сверху и одна снизу.

Формат записи `<element>:top=<lines>,bottom=<lines>`. Разделяйте элементы символом `;`, а стороны символом `,`. Пропущенные элементы и стороны сохраняют прежние отступы. Поддерживаемые типы блоков перечислены в [руководстве по разметке](../rendering/).

<strong class="cli-example-label">Пример</strong>

```sh
mdv --block-spacing 'paragraph:bottom=0;callout:top=2' example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="block-spacing"></div>

</details>

</section>

<section class="cli-option" aria-label="--horizontal-rule-style">

### `--horizontal-rule-style`

<p class="cli-syntax"><code>--horizontal-rule-style &lt;STYLE&gt;</code></p>

Управляет стилем разделителей Markdown и HTML `<hr>`.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`pretty`</strong><br> Рисует линию с декоративными символами по краям.

- `pretty` рисует линию с декоративными символами по краям.
- `simple` рисует сплошную линию.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --horizontal-rule-style simple example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="horizontal-rule-style"></div>

</details>

</section>

## Видимость элементов

<section class="cli-option" aria-label="--show-empty-elements">

### `--show-empty-elements`

Показывает пустые заголовки, элементы списков, цитаты, блоки кода и таблицы, которые иначе были бы скрыты.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Выключено</strong><br> Пустые элементы, лишние для разметки, скрываются.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --show-empty-elements example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="show-empty-elements"></div>

</details>

</section>

<section class="cli-option" aria-label="--no-code-guessing">

### `--no-code-guessing`

Отключает эвристическое определение языка. Если языковая метка распознана, подсветка продолжает работать.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">Определение включено</strong><br> Если метки нет или она неизвестна, mdv пытается определить язык блока.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --no-code-guessing example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="code-guessing"></div>

</details>

</section>

## Ссылки

<section class="cli-option" aria-label="--link-style">

### `--link-style`

<p class="cli-syntax"><code>-u, --link-style &lt;STYLE&gt;</code></p>

Задаёт способ отображения ссылок. Для гиперссылок OSC 8 требуется терминальное оформление.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`clickable`</strong><br> Показывает кликабельный текст ссылки в терминале с поддержкой OSC 8.

- `clickable` (`c`) делает текст ссылки кликабельным и скрывает URL.
- `fclickable` (`fc`) подчёркивает кликабельный текст ссылки.
- `inline` (`i`) показывает URL после текста ссылки.
- `inlinetable` (`it`) нумерует ссылки и показывает таблицу адресов рядом с блоком.
- `endtable` (`et`) нумерует ссылки и показывает таблицу адресов в конце документа.
- `hide` (`h`) оставляет только текст ссылки.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --link-style inline example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="link-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--link-overflow">

### `--link-overflow`

<p class="cli-syntax"><code>-l, --link-overflow &lt;MODE&gt;</code></p>

Определяет, что делать с видимым URL при нехватке ширины. Адрес перехода остаётся прежним.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`wrap`</strong><br> Переносит видимый URL, если он не помещается по ширине.

- `wrap` переносит длинные URL.
- `cut` обрезает длинные URL с многоточием.
- `tablecut` обрезает длинные URL в тексте и в ячейках таблиц.
- `none` разрешает выход за границы по горизонтали.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --link-style inline --link-overflow cut example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="link-overflow"></div>

</details>

</section>

## Сноски

<section class="cli-option" aria-label="--footnote-style">

### `--footnote-style`

<p class="cli-syntax"><code>--footnote-style &lt;STYLE&gt;</code></p>

Задаёт расположение сносок, на которые есть ссылки.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`endnotes`</strong><br> Собирает сноски, на которые есть ссылки, в конце документа.

- `endnotes` собирает сноски в конце документа.
- `attached` помещает сноски сразу после блока со ссылкой, например абзаца, таблицы или элемента списка.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --footnote-style attached example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="footnote-style"></div>

</details>

</section>

<section class="cli-option" aria-label="--missing-footnote-style">

### `--missing-footnote-style`

<p class="cli-syntax"><code>--missing-footnote-style &lt;STYLE&gt;</code></p>

Управляет отображением отсутствующих, некорректных и пустых определений сносок.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">`show`</strong><br> Показывает сообщение-заглушку для отсутствующих, некорректных или пустых определений.

- `show` выводит запись с сообщением-заглушкой.
- `hide` скрывает запись.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --missing-footnote-style hide example.md
```

<details class="mdv-reference-preview">
<summary>Пример отображения</summary>

<div data-mdv-preview="missing-footnote-style"></div>

</details>

</section>

## Справочная информация

<section class="cli-option" aria-label="mdv help">

### `mdv help`

Открывает полную справку в просмотрщике, если stdin и stdout подключены к терминалу. Просмотрщик выбирает `MDV_PAGER`, при отсутствии значения используется встроенный. Если один из потоков перенаправлен, справка печатается напрямую.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">По запросу</strong><br> Открывает справку при вызове подкоманды.

<strong class="cli-example-label">Пример</strong>

```sh
mdv help
```

</section>

<section class="cli-option" aria-label="-h">

### `-h`

Выводит краткую сводку параметров.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">По запросу</strong><br> Печатает сводку при указании флага.

<strong class="cli-example-label">Пример</strong>

```sh
mdv -h
```

</section>

<section class="cli-option" aria-label="--help">

### `--help`

Выводит полную справку с описаниями значений, форматами и примерами.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">По запросу</strong><br> Печатает полную справку при указании флага.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --help
```

</section>

<section class="cli-option" aria-label="--version">

### `--version`

<p class="cli-syntax"><code>-V, --version</code></p>

Выводит текущую версию.

<strong class="cli-default-label">По умолчанию</strong> <strong class="cli-default-value">По запросу</strong><br> Печатает версию при указании флага.

<strong class="cli-example-label">Пример</strong>

```sh
mdv --version
```

</section>
