import os
import time


def document(locale):
    if locale == 'ru':
        return '''# Руководство mdv

Читайте Markdown прямо в терминале.

## Установка

Установите mdv и откройте любой Markdown-файл.

```sh
cargo install mdv
mdv README.md
```

## Темы

Выберите тему для документа и интерфейса.

```sh
mdv --theme catppuccin README.md
```

### Пресеты

Пресет reader включает удобное оформление для чтения.

## Навигация

Используйте поиск, оглавление и номера строк.

- Найдите нужный раздел.
- Выделите текст для копирования.
- Вернитесь к чтению документа.

## Ссылки

Откройте [документацию](https://example.org/docs).
'''
    return '''# mdv guide

Read Markdown right in your terminal.

## Installation

Install mdv and open any Markdown document.

```sh
cargo install mdv
mdv README.md
```

## Themes

Choose a theme for the document and interface.

```sh
mdv --theme catppuccin README.md
```

### Presets

The reader preset provides comfortable reading defaults.

## Navigation

Use search, the outline and line numbers.

- Find the section you need.
- Select text to copy.
- Return to reading the document.

## Links

Open the [documentation](https://example.org/docs).
'''


def prepare(directory, locale):
    names = ['README.md', 'guides/installation.md', 'guides/themes.md', 'guides/presets.md',
             'guides/navigation.md', 'examples/code.md', 'examples/tables.md', 'CHANGELOG.md']
    timestamp = time.time() - 3600
    for index, name in enumerate(names):
        file = directory / name
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(document(locale), encoding='utf-8')
        os.utime(file, (timestamp - index * 3600, timestamp - index * 3600))
    return '\n'.join(['docs/', *['  ' + name for name in names]])


def states(kind, locale):
    def text(en, ru):
        return ru if locale == 'ru' else en

    def state(label, keys, steps, expected, en, ru):
        return {'label': label, 'keys': keys, 'steps': steps, 'expected': expected,
                'description': text(en, ru)}

    if kind == 'document-browser':
        return [
            state(text('Files', 'Файлы'), [], [], '8 documents',
                  'Choose a Markdown file. Enter opens it in the pager.',
                  'Выберите Markdown-файл. Enter открывает его в просмотрщике.'),
            state(text('Search', 'Поиск'), ['/', 'install'], ['/install'], 'installation.md',
                  'Press / and type part of a path. Matching characters are underlined.',
                  'Нажмите / и введите часть пути. Совпавшие символы подчёркиваются.'),
            state(text('Help', 'Справка'), ['?'], ['?'], 'close help',
                  'Press ? to show browser shortcuts.', 'Нажмите ?, чтобы показать управление выбором файлов.'),
        ]
    return [
        state(text('Document', 'Документ'), [], [], text('mdv guide', 'Руководство mdv'),
              'The document scrolls inside the terminal. The footer shows its position.',
              'Документ прокручивается внутри терминала. Нижняя панель показывает позицию.'),
        state(text('Search', 'Поиск'), ['/', 'mdv'], ['/mdv'], 'Find: mdv',
              'Search updates while you type and highlights individual matches.',
              'Поиск обновляется во время ввода и подсвечивает отдельные совпадения.'),
        state(text('Outline', 'Оглавление'), ['t'], ['t'], text('Installation', 'Установка'),
              'The outline shows sections beside the document; narrow terminals use an overlay.',
              'Оглавление показывает разделы рядом с документом, в узком терминале поверх него.'),
        state(text('Selection', 'Выделение'), ['Ctrl+A'], ['\x01'], text('Read Markdown', 'Читайте Markdown'),
              'Ctrl+A selects the entire rendered document, including text outside this screen.',
              'Ctrl+A выделяет весь оформленный документ, включая текст за пределами экрана.'),
        state(text('Line numbers', 'Номера строк'), ['l'], ['l'], text('mdv guide', 'Руководство mdv'),
              'Press l to cycle through off, rendered and source line numbers.',
              'Клавиша l переключает нумерацию: без номеров, строки вывода, строки исходника.'),
        state(text('Source line', 'Строка исходника'), [':', '16', 'Enter'], [':16\r'], ':16',
              'Jump to a physical Markdown source line. The target is highlighted.',
              'Перейдите к физической строке Markdown. Целевая строка подсвечивается.'),
        state(text('Help', 'Справка'), ['?'], ['?'], 'close help',
              'Press ? to see the controls available for this document.',
              'Нажмите ?, чтобы увидеть доступное управление для этого документа.'),
    ]
