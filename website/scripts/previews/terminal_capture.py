import queue
import re
import sys
import threading
import time

import pyte

if sys.platform == 'win32':
    from winpty import PtyProcess
else:
    from ptyprocess import PtyProcessUnicode as PtyProcess


class TerminalCapture:
    def __init__(self, command, directory, environment, columns=80, rows=22):
        self.process = PtyProcess.spawn(command, cwd=str(directory), env=environment,
                                        dimensions=(rows, columns))
        self.screen = pyte.Screen(columns, rows)
        self.screen.write_process_input = self.process.write
        self.stream = pyte.Stream(self.screen)
        self.chunks = queue.Queue()
        self.closed = False
        threading.Thread(target=self.read, daemon=True).start()

    def read(self):
        try:
            while not self.closed:
                self.chunks.put(self.process.read(65536))
        except (EOFError, OSError):
            if not self.closed:
                self.chunks.put(None)

    def wait(self, expected, timeout=8):
        deadline = time.monotonic() + timeout
        settled = None
        while time.monotonic() < deadline:
            try:
                chunk = self.chunks.get(timeout=0.05)
                if chunk is None:
                    raise RuntimeError('mdv exited before the requested terminal state.')
                self.stream.feed(chunk)
                settled = None
            except queue.Empty:
                text = '\n'.join(self.screen.display)
                if expected in text:
                    settled = settled or time.monotonic()
                    if time.monotonic() - settled > 0.3:
                        return
        raise RuntimeError(f'Terminal state not reached: {expected!r}\n' + '\n'.join(self.screen.display))

    def send(self, value):
        if sys.platform != 'win32':
            self.process.write(value)
            return
        # ConPTY requests Win32 input records so control keys retain their modifiers.
        for character in value:
            code = ord(character)
            control = 0 < code < 27 and character not in '\r\n\t'
            virtual = code + 64 if control else ord(character.upper())
            if character in '/?':
                virtual = 191
            elif character == ':':
                virtual = 186
            modifiers = 8 if control else 0
            unicode = code + 96 if control else code
            for pressed in (1, 0):
                self.process.write(f'\x1b[{virtual};0;{unicode};{pressed};{modifiers};1_')

    def ansi(self):
        lines = []
        for y in range(self.screen.lines):
            chunks = []
            previous = None
            for x in range(self.screen.columns):
                cell = self.screen.buffer[y][x]
                if not cell.data:
                    continue
                foreground, background = cell.fg, cell.bg
                if cell.reverse:
                    foreground, background = background, foreground
                    foreground = '#0b1017' if foreground == 'default' else foreground
                    background = '#cdd6f4' if background == 'default' else background
                style = (foreground, background, cell.bold, cell.italics, cell.underscore, cell.strikethrough)
                if style != previous:
                    codes = [0]
                    for color, offset in [(foreground, 30), (background, 40)]:
                        if color == 'default':
                            continue
                        if re.fullmatch(r'#?[a-fA-F0-9]{6}', color):
                            rgb = color.removeprefix('#')
                            codes.extend([offset + 8, 2, *[int(rgb[i:i + 2], 16) for i in (0, 2, 4)]])
                        else:
                            colors = ['black', 'red', 'green', 'brown', 'blue', 'magenta', 'cyan', 'white']
                            bright = color.startswith('bright')
                            codes.append(offset + colors.index(color.removeprefix('bright')) + (60 if bright else 0))
                    codes.extend(code for enabled, code in [(cell.bold, 1), (cell.italics, 3),
                                                            (cell.underscore, 4), (cell.strikethrough, 9)] if enabled)
                    chunks.append('\x1b[' + ';'.join(map(str, codes)) + 'm')
                    previous = style
                chunks.append(cell.data)
            lines.append(''.join(chunks) + '\x1b[0m')
        return '\n'.join(lines)

    def close(self):
        self.closed = True
        self.process.close(force=True)
