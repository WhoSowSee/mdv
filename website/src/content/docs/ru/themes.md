---
title: "Темы"
description: "Встроенные палитры, свои темы YAML, наследование и примеры оформления."
slug: "themes"
group: "Customization"
order: 30
---

## Выбор темы

Встроенные темы: `terminal`, `solarized-dark`, `nord`, `tokyonight`, `kanagawa`, `gruvbox`, `monokai`, `material-ocean`, `catppuccin`.

По умолчанию используется `terminal`. Код следует палитре документа, если другая палитра не выбрана через `--code-theme`. `--theme-info` показывает активные цвета; при переданном имени файла также отображает документ.

<div data-mdv-preview="theme"></div>

<div data-mdv-preview="code-theme"></div>

## Пользовательские темы

Положите один или несколько файлов `*.yaml`/`*.yml` в каталог `<config_dir>/themes/`, чтобы зарегистрировать собственные темы. Каталог ищется в том же порядке, что и `config.yaml` (флаг `--config-file`, переменная `MDV_CONFIG_PATH`, затем пользовательский каталог настроек ОС). В репозитории есть два готовых примера:

- [`docs/examples/themes/theme-warm.yaml`](https://github.com/WhoSowSee/mdv/blob/main/docs/examples/themes/theme-warm.yaml) - использует `extends: monokai` и переопределяет лишь несколько полей.
- [`docs/examples/themes/theme-custom.yaml`](https://github.com/WhoSowSee/mdv/blob/main/docs/examples/themes/theme-custom.yaml) - пример самостоятельной палитры.

```yaml
# <config_dir>/themes/warm.yaml
name: warm
description: Тёплый красный акцент поверх monokai.
extends: monokai
pager:
  transparent: true

h1: "#ff5577"
link: "#66ccff"
background: "#1c1c1c"

inline_style:
  code:
    backticks: false
    underline: true

syntax:
  keyword: "#ff5577"
```

Основные поля файла темы:

- `name` (обязательно) - значение, принимаемое параметром `--theme`.
- `description` (необязательно) - описание для `mdv --theme-info`; наследуется от базовой темы.
- `extends` (необязательно) - встроенная тема или пользовательская тема, загруженная раньше по алфавиту. Без него пропущенные поля наследуются от `terminal`.
- `pager` - цвета нижней панели, прозрачность, выделение и подсветка поиска во встроенном pager.
- `inline_style` - начертания строчных элементов Markdown; цвета задаются в соответствующих цветовых секциях.

Цвета наследуются по отдельным полям.

- Явный `null` очищает унаследованное необязательное переопределение цвета рамки или фона; `reset` использует цвет терминала по умолчанию.
- Полная схема и особые случаи описаны в [справочнике полей](../theme-reference/) и [правилах наследования](../theme-reference/).
- Все поля можно посмотреть в [файлах встроенных тем](https://github.com/WhoSowSee/mdv/blob/main/assets/config/themes/), а палитру по умолчанию - в [terminal.yaml](https://github.com/WhoSowSee/mdv/blob/main/assets/config/themes/terminal.yaml).

Пользовательская тема с тем же именем, что и встроенная, имеет приоритет и полностью её заменяет - это поддерживаемый способ сделать форк встроенной темы, не копируя каждое поле. Нечитаемые или повреждённые файлы пропускаются с предупреждением, поэтому одна битая тема не ломает остальные.
