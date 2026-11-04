---
title: "Разработка"
description: "Сборка, проверки и работа над mdv и его сайтом."
slug: "development"
group: "Reference"
order: 44
---

## Сборка и проверка

Порядок сообщения о проблемах, подготовки pull request и правила участия описаны в [CONTRIBUTING.md](https://github.com/WhoSowSee/mdv/blob/main/CONTRIBUTING.md).

При использовании AI для подготовки изменений прочитайте [AI Policy](https://github.com/WhoSowSee/mdv/blob/main/AI_POLICY.md).

Для локальной разработки [rust-toolchain.toml](https://github.com/WhoSowSee/mdv/blob/main/rust-toolchain.toml) выбирает стабильный Rust с `rustfmt` и `clippy`. Настройки форматирования находятся в [rustfmt.toml](https://github.com/WhoSowSee/mdv/blob/main/rustfmt.toml).

Проверки CI, сборка пакетов и публикация релизов описаны в [документации CI и релизов](https://github.com/WhoSowSee/mdv/blob/main/docs/modules/ci-release.md).

Необязательный [justfile](https://github.com/WhoSowSee/mdv/blob/main/justfile) содержит команды для разработки. Установите [just](https://just.systems/man/en/) версии 1.56.0 или новее командой `cargo install --locked just`.

```text
just --list
just check-all
just fmt
just run-dev README.md
```

На Windows нужен PowerShell 7.4+ (`pwsh.exe` в `PATH`); для `check-all` также требуется Python 3.11+. Все команды и подробности - в [документации разработки и тестирования](https://github.com/WhoSowSee/mdv/blob/main/docs/modules/testing.md).

### Сайт

Сайт проекта находится в [website/](https://github.com/WhoSowSee/mdv/blob/main/website/README.md) и использует Astro,
TypeScript и Tailwind CSS.

- Bun 1.4.4 или новее, минимальная версия указана в [website/package.json](https://github.com/WhoSowSee/mdv/blob/main/website/package.json).
- Для Astro также нужен Node.js 22.23.4 или новее в `PATH`.

```sh
cd website
bun install --frozen-lockfile
bun run dev
```

Откройте выведенный локальный адрес с путём `/mdv/`.

```sh
bun run test
bun run build
bun run preview
```

`bun run build` сначала обновляет ссылки на пакеты из последнего стабильного релиза
GitHub, затем проверяет типы и собирает `website/dist/`. Нужен доступ к GitHub API;
при необходимости задайте `GITHUB_TOKEN`. Режим разработки использует сохранённые данные релиза.

GitHub Actions проверяет pull request в `main` и публикует успешные сборки после
push в `main`. Сайт также обновляется после успешного workflow релиза по тегу.
Ручная публикация доступна через **Actions > Website > Run workflow** для `main`;
другие ветки только проверяются и собираются. Для публикации выберите
**Settings > Pages > Source > GitHub Actions**. Подробнее в
[workflow сайта](https://github.com/WhoSowSee/mdv/blob/main/.github/workflows/website.yml).

Сайт включает полные пользовательские руководства и справочники CLI/YAML по адресу
`/mdv/docs/`, с навигацией, полнотекстовым поиском и оглавлениями. Исходники Markdown находятся в
`website/src/content/docs/`.
Значок языка переключает текущую страницу между английским и русским.
Русская версия начинается с `/mdv/ru/`, документация находится в `/mdv/ru/docs/`.
