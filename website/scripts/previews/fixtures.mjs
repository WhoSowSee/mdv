export function fixtures(locale) {
  const t = (en, ru) => locale === 'en' ? en : ru;
  const paragraph = t('A Markdown viewer keeps a long paragraph readable while you change the terminal width. Each option controls how these words are arranged without changing the document.', 'Просмотрщик Markdown сохраняет читаемость длинного абзаца при изменении ширины терминала. Каждый параметр управляет расположением слов без изменения самого документа.');
  const code = '```rust\nfn main() {\n    println!("hello, mdv");\n}\n```';
  const table = '| Name | Description | Status |\n| --- | --- | --- |\n| reader | Comfortable long-form reading with word wrapping | ready |\n| compact | Dense output for a narrow terminal workspace | ready |';
  return {
    links: t('Open the [guide](https://example.org/guide) and [examples](https://example.org/examples).\n\nKeep reading the document.', 'Откройте [руководство](https://example.org/guide) и [примеры](https://example.org/examples).\n\nПродолжайте чтение документа.'),
    'long-link': '[Documentation](https://example.org/documentation/terminal/formatting/links-and-footnotes)\n\n| Page | Link |\n| --- | --- |\n| Guide | [Open](https://example.org/documentation/terminal/formatting/links-and-footnotes) |',
    footnotes: t('First statement.[^a]\n\nAnother paragraph.\n\nSecond statement.[^b]\n\n[^a]: First note.\n[^b]: Second note.', 'Первое утверждение.[^a]\n\nДругой абзац.\n\nВторое утверждение.[^b]\n\n[^a]: Первая сноска.\n[^b]: Вторая сноска.'),
    'missing-footnote': t('A statement with a missing note.[^missing]', 'Утверждение с отсутствующей сноской.[^missing]'),
    paragraph,
    'hard-wrapped': t('This paragraph was wrapped\nin the source file.\nThe reader can join its\nsoft line breaks together.', 'Этот абзац перенесён\nв исходном файле.\nПросмотрщик может объединить\nего мягкие переводы строк.'),
    headings: t('# Guide\n\nAn introduction.\n\n## Installation\n\nInstall the program.\n\n### Usage\n\nOpen a document.', '# Руководство\n\nВведение.\n\n## Установка\n\nУстановите программу.\n\n### Использование\n\nОткройте документ.'),
    'heading-jump': t('# Guide\n\nIntroduction.\n\n#### Details\n\nA level was skipped.', '# Руководство\n\nВведение.\n\n#### Подробности\n\nПропущен уровень заголовка.'),
    table,
    'small-table': '| Name | Wrap |\n| --- | --- |\n| reader | word |\n| compact | char |',
    'indented-table': '###### Settings\n\n'+table,
    spacing: t('## First section\n\nFirst paragraph.\n\nSecond paragraph.\n\n## Next section\n\nKeep reading.', '## Первый раздел\n\nПервый абзац.\n\nВторой абзац.\n\n## Следующий раздел\n\nПродолжайте чтение.'),
    rule: t('Before the rule.\n\n---\n\nAfter the rule.', 'До разделителя.\n\n---\n\nПосле разделителя.'),
    comments: t('Visible text.\n\n<!-- An internal comment. -->\n\nMore text.', 'Видимый текст.\n\n<!-- Внутренний комментарий. -->\n\nДругой текст.'),
    empty: t('Before an empty code block.\n\n```rust\n```\n\nAfter the block.', 'Перед пустым блоком кода.\n\n```rust\n```\n\nПосле блока.'),
    numbered: t('## Example\n\nA paragraph long enough to wrap into multiple rendered rows at the selected terminal width.\n\nLast paragraph.', '## Пример\n\nДлинный абзац переносится на несколько строк вывода при выбранной ширине терминала.\n\nПоследний абзац.'),
    code,
    tabs: '```text\nname\tstatus\nmdv\tready\n```',
    'code-long': '```rust\nfn main() {\n    let message = "A long line of code that wraps to the available terminal width";\n    println!("{message}");\n}\n```',
    'unknown-code': '```\n#!/usr/bin/env python3\ndef greet(name):\n    print(f"Hello, {name}!")\ngreet("mdv")\n```',
    math: '$$\n\\frac{a+b}{c} = \\sqrt{x^2 + 1}\n$$',
    callout: t('> [!NOTE]\n> Read Markdown in your terminal.\n\n> [!TIP]\n> Choose a theme for your workspace.', '> [!NOTE]\n> Читайте Markdown в терминале.\n\n> [!TIP]\n> Выберите тему для работы.'),
    'folded-callout': t('> [!NOTE]- Details\n> Terminal content remains visible.', '> [!NOTE]- Подробности\n> Содержимое остаётся видимым.'),
    tasks: t('- [ ] Read the guide\n- [x] Install mdv', '- [ ] Прочитать руководство\n- [x] Установить mdv'),
    'custom-tasks': t('- [ ] Read the guide\n- [x] Install mdv\n- [*] Review settings\n- [?] Ask a question', '- [ ] Прочитать руководство\n- [x] Установить mdv\n- [*] Проверить настройки\n- [?] Задать вопрос'),
    lists: t('- Documents\n  - Guides\n    - Installation\n      - Linux\n- Examples', '- Документы\n  - Руководства\n    - Установка\n      - Linux\n- Примеры'),
    definitions: t('Theme\n: Colors for a document.\n\nPreset\n: A reusable group of settings.', 'Тема\n: Цвета документа.\n\nПресет\n: Группа готовых настроек.'),
    'front-matter': t('---\ntitle: Guide\nauthor: Alice\ntags: [docs, terminal]\n---\n\n## Start\n\nRead the document.', '---\ntitle: Руководство\nauthor: Alice\ntags: [docs, terminal]\n---\n\n## Начало\n\nЧитайте документ.'),
    html: t('<details>\n<summary>Details</summary>\n<p>A <strong>formatted</strong> paragraph.</p>\n</details>', '<details>\n<summary>Подробности</summary>\n<p><strong>Оформленный</strong> абзац.</p>\n</details>'),
    theme: t('# Example\n\nRead **Markdown** with `mdv`.\n\n> [!NOTE]\n> Terminal document preview.\n\n'+code+'\n\nOpen the [guide](https://example.org/guide).', '# Пример\n\nЧитайте **Markdown** через `mdv`.\n\n> [!NOTE]\n> Пример документа в терминале.\n\n'+code+'\n\nОткройте [руководство](https://example.org/guide).'),
    inline: t('*Emphasis*, **strong text**, ***both***.\n\nInline `code`, ~~removed~~ and ==highlighted== text.', '*Курсив*, **жирный текст**, ***оба***.\n\nСтрочный `код`, ~~удалённый~~ и ==выделенный== текст.'),
  };
}
