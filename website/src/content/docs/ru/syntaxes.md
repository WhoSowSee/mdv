---
title: "Свои синтаксисы"
description: "Загрузка определений Sublime Syntax и настройка алиасов языка."
slug: "syntaxes"
group: "Customization"
order: 33
---

## Загрузка определений

`--syntaxes-dir DIR` рекурсивно загружает файлы `.sublime-syntax` поверх встроенного набора. Пользовательские определения важнее совпадающих встроенных. Отсутствующий каталог или некорректное определение приводят к ошибке, а не к незаметному переходу к простому тексту.

```sh
mdv --syntaxes-dir ./syntaxes README.md
```

## Минимальный пример

Сохраните файл `syntaxes/Acme.sublime-syntax`:

```yaml
%YAML 1.2
---
name: Acme
file_extensions: [acme]
scope: source.acme
contexts:
  main:
    - match: '\b(let|print)\b'
      scope: keyword.control.acme
```

Укажите `acme` как язык блока кода Markdown. Значки, имена и дополнительные алиасы задаются отдельно:

```sh
mdv --syntaxes-dir ./syntaxes --custom-code-block 'acme:label=Acme,icon=*,aliases=ac' README.md
```

## Пути конфигурации и определение языка

`syntaxes_dir` сохраняет каталог в YAML. Относительные пути YAML отсчитываются от каталога конфигурации, а относительный путь CLI от текущего рабочего каталога. Пользовательские алиасы влияют на поиск, но одни подписи блоков не добавляют грамматику. `--no-code-guessing` отключает эвристическое определение, сохраняя явный выбор синтаксиса.
