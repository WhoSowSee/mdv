const text = (en, ru) => ({ en, ru });
const choice = (label, args, labels) => ({ label: labels ?? text(label, label), args });
const options = (flag, values) => values.map((value) => choice(value, [flag, value]));
const defaultChoice = (args = []) => choice('default', args, text('Default', 'По умолчанию'));
const toggle = (flag, extra = []) => [choice('off', extra, text('Off', 'Выкл.')), choice('on', [flag, ...extra], text('On', 'Вкл.'))];
const modifiers = (flag, base, values) => [defaultChoice([flag, base]), ...values.map((value) => choice(value, [flag, `${base}${base.includes(':') ? ';' : ':'}${value}`]))];
const lineNumbers = (flag) => [defaultChoice(), choice(flag, [flag]), ...options(flag, ['source', 'separator', 'source;separator'])];
const spec = (id, page, heading, title, fixture, variants, extra = {}) => ({ id, page, heading, title, fixture, variants, ...extra });

export const catalog = [
  spec('link-style', 'links', 0, text('Link presentation', 'Отображение ссылок'), 'links', options('--link-style', ['clickable', 'fclickable', 'inline', 'inlinetable', 'endtable', 'hide'])),
  spec('link-overflow', 'links', 1, text('Long URLs', 'Длинные адреса'), 'long-link', options('--link-overflow', ['wrap', 'cut', 'tablecut', 'none']), { base: ['--link-style', 'inline', '--cols', '40'] }),
  spec('footnote-style', 'links', 2, text('Footnote placement', 'Размещение сносок'), 'footnotes', options('--footnote-style', ['endnotes', 'attached'])),
  spec('missing-footnote-style', 'links', 3, text('Missing definitions', 'Отсутствующие определения'), 'missing-footnote', options('--missing-footnote-style', ['show', 'hide'])),
  spec('cols', 'rendering', 0, text('Output width', 'Ширина вывода'), 'paragraph', ['40', '56', '72'].map((n) => choice(n, ['--cols', n, '--wrap', 'word']))),
  spec('margin', 'rendering', 0, text('Horizontal margins', 'Горизонтальные поля'), 'paragraph', [defaultChoice(), ...options('--margin', ['left:4', 'left:4;right:4'])]),
  spec('tab-length', 'rendering', 0, text('Tab expansion', 'Ширина табуляции'), 'tabs', options('--tab-length', ['2', '4', '8'])),
  spec('wrap', 'rendering', 1, text('Line wrapping', 'Перенос строк'), 'paragraph', options('--wrap', ['char', 'word', 'none']), { base: ['--cols', '40'] }),
  spec('reflow', 'rendering', 1, text('Paragraph reflow', 'Переформатирование абзаца'), 'hard-wrapped', toggle('--reflow', ['--wrap', 'word'])),
  spec('heading-layout', 'rendering', 2, text('Heading layout', 'Разметка заголовков'), 'headings', options('--heading-layout', ['level', 'center', 'flat', 'none'])),
  spec('show-heading-markers', 'rendering', 2, text('Heading markers', 'Маркеры заголовков'), 'headings', toggle('--show-heading-markers')),
  spec('smart-indent', 'rendering', 2, text('Heading level jumps', 'Скачки уровней заголовков'), 'heading-jump', toggle('--smart-indent')),
  spec('table-wrap', 'rendering', 3, text('Wide tables', 'Широкие таблицы'), 'table', options('--table-wrap', ['fit', 'wrap', 'none']), { base: ['--cols', '40'] }),
  spec('table-borders', 'rendering', 3, text('Table borders', 'Рамки таблиц'), 'small-table', toggle('--table-borders')),
  spec('table-smart-indent', 'rendering', 3, text('Table indentation', 'Отступ таблицы'), 'indented-table', toggle('--table-smart-indent'), { base: ['--cols', '40'] }),
  spec('block-spacing', 'rendering', 4, text('Block spacing', 'Интервалы между блоками'), 'spacing', [defaultChoice(), ...options('--block-spacing', ['paragraph:top=0,bottom=0;h2:top=0,bottom=0', 'paragraph:top=1,bottom=2;h2:top=2,bottom=1'])]),
  spec('horizontal-rule-style', 'rendering', 5, text('Horizontal rules', 'Горизонтальные линии'), 'rule', options('--horizontal-rule-style', ['pretty', 'simple'])),
  spec('hide-comments', 'rendering', 5, text('Comment visibility', 'Видимость комментариев'), 'comments', toggle('--hide-comments')),
  spec('show-empty-elements', 'rendering', 5, text('Empty blocks', 'Пустые блоки'), 'empty', toggle('--show-empty-elements')),
  spec('line-numbers', 'rendering', 6, text('Document line numbers', 'Номера строк документа'), 'numbered', lineNumbers('--line-numbers')),
  spec('code-block-style', 'code-blocks', 0, text('Code block frame', 'Оформление блока кода'), 'code', options('--code-block-style', ['basic', 'simple', 'pretty'])),
  spec('code-labels', 'code-blocks', 0, text('Code labels and icons', 'Подписи и значки кода'), 'code', modifiers('--code-block-style', 'pretty', ['show-name', 'show-icon', 'show-name;show-icon']), { option: '--code-block-style', description: text('Hidden labels, a language name, an icon, or both.', 'Скрытые подписи, название языка, значок или оба.') }),
  spec('code-line-numbers', 'code-blocks', 1, text('Code line numbers', 'Номера строк кода'), 'code-long', lineNumbers('--code-line-numbers'), { base: ['--cols', '44'] }),
  spec('code-wrap-indent', 'code-blocks', 2, text('Code continuation indent', 'Отступ продолжений кода'), 'code-long', options('--code-wrap-indent', ['none', 'base', 'double']), { base: ['--cols', '44'] }),
  spec('code-guessing', 'code-blocks', 3, text('Language guessing', 'Определение языка'), 'unknown-code', [choice('on', [], text('On', 'Вкл.')), choice('off', ['--no-code-guessing'], text('Off', 'Выкл.'))], { option: '--no-code-guessing' }),
  spec('custom-code-block', 'code-blocks', 4, text('Custom code labels', 'Свои подписи кода'), 'code', [defaultChoice(), choice('--custom-code-block', ['--custom-code-block', 'rust:label=Example,icon=*'])], { base: ['--code-block-style', 'pretty:show-name;show-icon'] }),
  spec('math-block-style', 'math', 1, text('Math block frame', 'Оформление формулы'), 'math', options('--math-block-style', ['basic', 'simple', 'pretty'])),
  spec('callout-style', 'callouts', 0, text('Callout layout', 'Оформление выноски'), 'callout', options('--callout-style', ['pretty', 'simple'])),
  spec('callout-icons', 'callouts', 0, text('Callout icons', 'Значки выносок'), 'callout', modifiers('--callout-style', 'pretty', ['show-simple-icons', 'show-icons']), { option: '--callout-style', description: text('No icons, ASCII markers, or Nerd Font icons.', 'Без значков, ASCII-маркеры или значки Nerd Font.') }),
  spec('callout-label', 'callouts', 0, text('Callout title position', 'Положение подписи'), 'callout', modifiers('--callout-style', 'pretty', ['label-inside']), { option: '--callout-style' }),
  spec('callout-uppercase', 'callouts', 0, text('Callout title case', 'Регистр подписи'), 'callout', modifiers('--callout-style', 'pretty', ['uppercase']), { option: '--callout-style' }),
  spec('callout-fold', 'callouts', 1, text('Fold indicator', 'Индикатор сворачивания'), 'folded-callout', modifiers('--callout-style', 'pretty:show-icons', ['fold-icons']), { option: '--callout-style' }),
  spec('custom-callout', 'callouts', 2, text('Custom callout color', 'Свой цвет выноски'), 'callout', [defaultChoice(), choice('--custom-callout', ['--custom-callout', 'note:icon=*,color=#fab387;tip:color=#89b4fa'])]),
  spec('checkbox-style', 'lists', 0, text('Task-list checkboxes', 'Чекбоксы задач'), 'tasks', [defaultChoice(), ...options('--checkbox-style', ['square', 'circle'])]),
  spec('custom-checkbox', 'lists', 0, text('Custom task states', 'Свои состояния задач'), 'custom-tasks', [defaultChoice(), choice('--custom-checkbox', ['--custom-checkbox', '*:icon=*,color=yellow;?:color=red'])], { base: ['--checkbox-style', 'square'] }),
  spec('list-style', 'lists', 1, text('Nested list markers', 'Маркеры вложенных списков'), 'lists', [defaultChoice(), ...options('--list-style', ['type:unicode;size:small', 'type:nerd-font;size:small', 'type:nerd-font;size:large'])]),
  spec('uniform-list-marker', 'lists', 1, text('Uniform markers', 'Единый маркер'), 'lists', [defaultChoice(), ...options('--uniform-list-marker', ['level:2', 'icon:*'])], { base: ['--list-style', 'type:unicode;size:small'] }),
  spec('custom-list', 'lists', 1, text('Custom list colors', 'Свои цвета списка'), 'lists', [defaultChoice(), choice('--custom-list', ['--custom-list', '1:icon=*,color=yellow;2:color=red'])], { base: ['--list-style', 'type:unicode;size:small'] }),
  spec('definition-marker-style', 'lists', 1, text('Definition markers', 'Маркеры определений'), 'definitions', [defaultChoice(), ...options('--definition-marker-style', ['unicode', 'nerd-font'])]),
  spec('front-matter', 'front-matter', 1, text('Front matter display', 'Отображение свойств'), 'front-matter', options('--front-matter', ['hidden', 'panel', 'table', 'plain', 'inline', 'blocks', 'code', 'source'])),
  spec('render-html', 'html', 1, text('Embedded HTML', 'Встроенный HTML'), 'html', toggle('--render-html')),
  spec('theme', 'themes', 0, text('Document palette', 'Палитра документа'), 'theme', options('--theme', ['terminal', 'solarized-dark', 'nord', 'tokyonight', 'kanagawa', 'gruvbox', 'monokai', 'material-ocean', 'catppuccin'])),
  spec('code-theme', 'themes', 0, text('Independent code palette', 'Отдельная палитра кода'), 'code', options('--code-theme', ['terminal', 'nord', 'gruvbox', 'tokyonight', 'catppuccin'])),
  spec('custom-theme', 'theme-reference', 1, text('Theme color overrides', 'Переопределение цветов темы'), 'theme', [defaultChoice(), ...options('--custom-theme', ['h1=#89b4fa;link=#89b4fa', 'h1=#fab387;link=#fab387'])]),
  spec('custom-code-theme', 'theme-reference', 1, text('Syntax color overrides', 'Переопределение цветов синтаксиса'), 'code', [defaultChoice(), choice('--custom-code-theme', ['--custom-code-theme', 'keyword=#89b4fa;string=#fab387'])]),
  spec('inline-style', 'theme-reference', 3, text('Inline decorations', 'Строчное оформление'), 'inline', [defaultChoice(), ...options('--inline-style', ['code:backticks=false', 'code:backticks=false,underline=true;emphasis:italic=false,bold=true'])]),
  spec('preset', 'presets', 0, text('Reading presets', 'Пресеты чтения'), 'theme', [defaultChoice(), ...options('--preset', ['reader', 'compact', 'pretty', 'showcase'])]),
  spec('color-depth', 'cli', null, text('Terminal color depth', 'Глубина цвета терминала'), 'theme', options('--color-depth', ['16', '256', 'truecolor'])),
  spec('color', 'cli', null, text('Terminal styling', 'Терминальное оформление'), 'inline', options('--color', ['always', 'never'])),
  spec('reverse', 'usage', 4, text('Reading direction', 'Направление чтения'), 'spacing', toggle('--reverse')),
];

for (const entry of catalog) entry.option ??= `--${entry.id}`;
