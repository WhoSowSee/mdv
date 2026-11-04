---
title: "Установка"
description: "Установка, обновление и сборка mdv на Windows, Linux и macOS."
slug: "installation"
group: "Getting started"
order: 1
---

## Установка одной командой

Установщики скачивают последний релиз GitHub для вашей системы и архитектуры, проверяют SHA-256 и устанавливают исполняемый файл. Rust не нужен. Повторите ту же команду для обновления.

### Windows

Запустите в Windows PowerShell 5.1 или PowerShell 7:

```powershell
irm https://whosowsee.github.io/mdv/install.ps1 | iex
```

Поддерживаются x64 и ARM64. Файл устанавливается в `%LOCALAPPDATA%\Programs\mdv`, каталог добавляется в пользовательский `PATH` и текущую сессию PowerShell. Права администратора не нужны.

### Linux и macOS

```sh
curl -fsSL https://whosowsee.github.io/mdv/install.sh | sh
```

Поддерживаются x86_64 и ARM64 на Linux, Intel и Apple Silicon на macOS. Нужны `curl`, `unzip` и системная утилита SHA-256 (`sha256sum` на Linux, `shasum` на macOS). Для Linux используется статическая musl-сборка. Каталог установки: `~/.local/bin`. Профили оболочек не изменяются.

Если каталога нет в `PATH`, добавьте следующую строку в профиль оболочки и выполните её в текущей сессии:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

### Выбор версии и каталога

Передайте тег релиза сразу при запуске установщика. Каталог установки указывать необязательно:

Linux и macOS:

```sh
curl -fsSL https://whosowsee.github.io/mdv/install.sh | sh -s -- --version 'v<VERSION>'
```

Windows:

```powershell
& ([scriptblock]::Create((irm https://whosowsee.github.io/mdv/install.ps1))) -Version 'v<VERSION>'
```

Замените `v<VERSION>` точным тегом релиза.

- Без параметра версии устанавливается последний релиз.
- Если путь не указан, используется стандартный каталог, описанный выше.
- Для своего каталога добавьте `--install-dir "/path/to/bin"` к команде sh или `-InstallDir "C:\Tools\mdv"` к команде PowerShell.
- Переменная `MDV_INSTALL_DIR` также задаёт каталог для sh-установщика.
- Необязательный `-NoModifyPath` в PowerShell сохраняет пользовательский PATH и PATH текущей сессии.

### Скачивание перед запуском

Чтобы просмотреть скрипт до выполнения, сохраните его локально и передайте те же параметры:

```sh
curl -fsSL https://whosowsee.github.io/mdv/install.sh -o install.sh
sh install.sh --version 'v<VERSION>'
```

```powershell
irm https://whosowsee.github.io/mdv/install.ps1 -OutFile install.ps1
& ([scriptblock]::Create((Get-Content -Raw ./install.ps1))) -Version 'v<VERSION>'
```

Если нет архива, контрольной суммы или проверка не прошла, существующий бинарник не заменяется. Скрипты не устанавливают автодополнение и ассоциации файлов. Для удаления уберите установленный бинарник и добавленную запись PATH.

## Установка через Cargo

Если установлены Rust 1.95 или новее и Cargo:

```sh
cargo install mdv
```

Cargo соберёт опубликованную версию и поместит исполняемый файл в свой каталог программ. Убедитесь, что этот каталог добавлен в `PATH`.

## Готовые файлы

Скачайте архив для своей операционной системы и архитектуры из [GitHub Releases](https://github.com/WhoSowSee/mdv/releases). Распакуйте его и поместите исполняемый файл в каталог из `PATH`.

Для готовых файлов Rust не нужен. Релизы также содержат пакеты Linux. Выбирайте пакет для своей системы и архитектуры. Пакеты RPM требуют glibc 2.39 или новее.

## Установка RPM и пакетов Arch Linux

Команды скачивают пакет из [GitHub Releases](https://github.com/WhoSowSee/mdv/releases) в текущий каталог и устанавливают его. Нужен `curl`. Выполните строку установки после успешного скачивания. Можно также скачать пакет вручную и выполнить только вторую строку.

Для Fedora и совместимых RPM-систем (x86_64):

```sh
curl -fLO "https://github.com/WhoSowSee/mdv/releases/download/v<VERSION>/mdv-v<VERSION>-x86_64-unknown-linux-gnu.rpm"
sudo dnf install "./mdv-v<VERSION>-x86_64-unknown-linux-gnu.rpm"
```

Для Arch Linux (x86_64):

```sh
curl -fLO "https://github.com/WhoSowSee/mdv/releases/download/v<VERSION>/mdv-v<VERSION>-x86_64-unknown-linux-gnu.pkg.tar.zst"
sudo pacman -U "./mdv-v<VERSION>-x86_64-unknown-linux-gnu.pkg.tar.zst"
```

Замените все `<VERSION>` номером релиза.

- Для RPM на ARM64 замените `x86_64` на `aarch64` в обеих строках.
- Оба формата требуют glibc 2.39 или новее; пакеты Arch Linux ARM не публикуются.
- Rust не нужен.
- Установка локального пакета не подключает репозиторий обновлений; для обновления установите пакет нового релиза.

На главной странице указаны точные имена файлов и ссылки на пакеты последнего релиза, проверенного при сборке сайта. Эти команды не используют shell-установщик.

## Установка через Nix

Если установлен Nix и включены flakes, можно попробовать mdv без постоянной установки:

```sh
nix run github:WhoSowSee/mdv
```

Или установить в свой профиль:

```sh
nix profile install github:WhoSowSee/mdv
```

Flake поддерживает x86_64 и ARM64 Linux.

## Сборка из исходников

Нужны Git и Rust 1.95 или новее:

```sh
git clone https://github.com/WhoSowSee/mdv.git
cd mdv
cargo install --path .
```

## Проверка установки

```sh
mdv --version
mdv README.md
```

Используйте терминал с поддержкой цветов ANSI. Nerd Font нужен только для функций, которые отображают его значки.

## Запуск без установки

```bash
cargo build --release
./target/release/mdv README.md
```

Можно запускать бинарник из `target/release` напрямую или добавить путь к нему в `PATH`.
