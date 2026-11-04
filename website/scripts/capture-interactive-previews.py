import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

from previews.interactive_fixtures import document, prepare, states
from previews.terminal_capture import TerminalCapture


def capture(binary):
    project = Path(__file__).resolve().parents[2]
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith('MDV_') and key not in {'NO_COLOR', 'FORCE_COLOR', 'CLICOLOR', 'CLICOLOR_FORCE'}}
    environment.update(TERM='xterm-256color', COLORTERM='truecolor')
    build = subprocess.check_output([str(binary), '--version'], text=True, env=environment).strip()
    version = re.search(r'Version:\s*(\S+)', build)[1]
    expected = re.search(r'^version\s*=\s*"([^"]+)"', (project / 'Cargo.toml').read_text(), re.M)[1]
    if version != expected:
        raise ValueError('Build mdv from the current project before capturing previews.')
    previews = {}
    for kind in ['document-browser', 'builtin-pager']:
        preview = {
            'title': {'en': 'Document browser' if kind == 'document-browser' else 'Built-in pager',
                      'ru': 'Выбор документов' if kind == 'document-browser' else 'Встроенный просмотрщик'},
            'sourceLabel': {'en': 'Example files' if kind == 'document-browser' else 'Markdown source',
                            'ru': 'Файлы примера' if kind == 'document-browser' else 'Исходный Markdown'},
        }
        for locale in ['en', 'ru']:
            with tempfile.TemporaryDirectory(prefix='mdv-interactive-preview-') as scratch:
                directory = Path(scratch)
                if directory.resolve().parent != Path(tempfile.gettempdir()).resolve():
                    raise ValueError('Unexpected temporary directory.')
                tree = prepare(directory, locale)
                source = tree if kind == 'document-browser' else document(locale)
                args = ['--no-config', '--config-file', str(directory), '--theme', 'catppuccin',
                        '--color', 'always', '--color-depth', 'truecolor']
                args += ['-i', '.'] if kind == 'document-browser' else ['--wrap', 'word', '--pager=builtin', 'README.md']
                command = 'mdv --no-config --theme catppuccin '
                command += '-i docs/' if kind == 'document-browser' else '--wrap word --pager=builtin README.md'
                variants = []
                for state in states(kind, locale):
                    terminal = TerminalCapture([str(binary), *args], directory, environment)
                    try:
                        initial = '8 documents' if kind == 'document-browser' else ('Руководство mdv' if locale == 'ru' else 'mdv guide')
                        terminal.wait(initial)
                        for step in state['steps']:
                            terminal.send(step)
                        terminal.wait(state['expected'])
                        variants.append({key: state[key] for key in ['label', 'keys', 'description']} | {
                            'command': command, 'ansi': terminal.ansi(), 'rows': 22, 'columns': 80,
                        })
                    finally:
                        terminal.close()
                preview[locale] = {'source': source, 'variants': variants}
                if len({variant['ansi'] for variant in variants}) != len(variants):
                    duplicates = [(a['label'], b['label']) for index, a in enumerate(variants)
                                  for b in variants[index + 1:] if a['ansi'] == b['ansi']]
                    raise ValueError(f'Duplicate interactive states: {kind} ({locale}): {duplicates}')
                print(f'{kind} ({locale}): {len(variants)} terminal states')
        previews[kind] = preview
    return {'schema': 1, 'version': version, 'build': build, 'previews': previews}


if __name__ == '__main__':
    project = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description='Capture real mdv TUI states. Install scripts/preview-requirements.txt first.')
    parser.add_argument('binary', nargs='?', type=Path, default=project / 'target/release' / ('mdv.exe' if os.name == 'nt' else 'mdv'))
    parser.add_argument('--check', action='store_true')
    arguments = parser.parse_args()
    result = capture(arguments.binary.resolve())
    destination = project / 'website/src/data/interactive-previews.json'
    if arguments.check:
        if result != json.loads(destination.read_text(encoding='utf-8')):
            raise ValueError('Interactive previews differ from current mdv output. Regenerate the snapshot.')
        print('Every interactive state matches current mdv output in both languages.')
    else:
        destination.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
