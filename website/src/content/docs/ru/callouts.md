---
title: "Выноски"
description: "Типы выносок, стили, значки, пользовательские определения и YAML."
slug: "callouts"
group: "Formatting"
order: 23
---

## Стиль и параметры

- `-C, --callout-style <pretty|simple>[:show-icons;show-simple-icons;fold-icons;label-inside;uppercase]` - стиль callout-блоков и правила отображения подписи. `show-icons` использует глифы Nerd Font, а `show-simple-icons` - переносимые ASCII-маркеры вроде `[i]`, `[*]`, `[!]` и `[x]`. Эти два параметра нельзя комбинировать.
  `label-inside` поддерживается только с `pretty`, `fold-icons` требует `show-icons`.
- `--custom-callout <name:icon=...,color=...;...>` - переопределение или добавление подписей callout.
  `icon` и `color` можно указывать по отдельности; формат цвета как в `--custom-theme`.
- Для корректного отображения иконок, включённых через `show-icons` и `fold-icons`, нужны шрифты Nerd Fonts в терминале. Для `show-simple-icons` они не требуются.

<div data-mdv-preview="callout-style"></div>

<div data-mdv-preview="callout-icons"></div>

<div data-mdv-preview="callout-label"></div>

<div data-mdv-preview="callout-uppercase"></div>

## Исходный Markdown

```markdown
> [!NOTE]
> Keep related information together.

> [!WARNING]
> Check the selected destination before continuing.
```

Встроенные типы: `note`, `abstract`, `info`, `todo`, `tip`, `success`, `question`, `warning`, `failure`, `danger`, `bug`, `example`, `quote`. Индикаторы сворачивания относятся к оформлению; содержимое в терминале остаётся видимым.

<div data-mdv-preview="callout-fold"></div>

## Примеры

<div data-mdv-preview="custom-callout"></div>

## YAML и приоритет цветов

```yaml
callout_style:
  style: pretty
  show_simple_icons: true
  label_inside: true
  uppercase: false
custom_callout:
  tip:
    icon: "*"
    color: green
  custom:
    color: "#89b4fa"
```

Пользовательские определения объединяются по имени и свойствам. Переопределение только цвета сохраняет значок. `null` очищает свойство или запись, корневые `null` и `{}` очищают всю настройку. В CLI также доступны `tip:color=null`, `tip:null` и `--custom-callout null`.

При одинаковом приоритете источника цвет типа из пользовательского определения важнее общего `callout.label`, затем `callout.palette`. Цвета CLI выше по приоритету, чем цвета конфигурации. Рамка использует отдельный `callout.border`; см. [справочник полей темы](../theme-reference/).
