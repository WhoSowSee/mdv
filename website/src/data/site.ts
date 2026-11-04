import packageRelease from './package-release.json';

export const links = {
  github: 'https://github.com/WhoSowSee/mdv',
  releases: 'https://github.com/WhoSowSee/mdv/releases',
  crate: 'https://crates.io/crates/mdv',
  license: 'https://github.com/WhoSowSee/mdv/blob/main/LICENSE',
};

export const palettes = [
  { name: 'nord', colors: ['#2e3440', '#88c0d0', '#a3be8c', '#ebcb8b', '#bf616a'] },
  { name: 'gruvbox', colors: ['#282828', '#fabd2f', '#b8bb26', '#83a598', '#fb4934'] },
  { name: 'tokyonight', colors: ['#1a1b26', '#7aa2f7', '#bb9af7', '#9ece6a', '#f7768e'] },
  { name: 'catppuccin', colors: ['#1e1e2e', '#cba6f7', '#a6e3a1', '#f9e2af', '#f38ba8'] },
];

interface InstallMethod {
  id: string;
  label: string;
  command: string;
  note: string;
  downloads?: readonly { label: string; href: string }[];
}

const { arch, rpm, rpmArm64 } = packageRelease.packages;
export const cargoInstallCommand = 'cargo install mdv';

export const usageCommands = {
  file: 'mdv README.md',
  monitor: 'mdv --monitor README.md',
  browser: 'mdv -i',
  stdin: 'cat notes.md | mdv',
  preset: 'mdv --preset reader README.md',
};

const installCommands = [
  { id: 'cargo', label: 'CARGO', command: cargoInstallCommand },
  {
    id: 'windows', label: 'WINDOWS',
    command: 'irm https://whosowsee.github.io/mdv/install.ps1 | iex',
  },
  {
    id: 'unix', label: 'LINUX / MACOS',
    command: 'curl -fsSL https://whosowsee.github.io/mdv/install.sh | sh',
  },
  {
    id: 'arch', label: 'ARCH',
    command: `curl -fLO "${arch.url}"\nsudo pacman -U "./${arch.filename}"`,
    downloads: [{ label: 'Arch Linux x86_64', href: arch.url }],
  },
  {
    id: 'rpm', label: 'RPM',
    command: `curl -fLO "${rpm.url}"\nsudo dnf install "./${rpm.filename}"`,
    downloads: [
      { label: 'RPM x86_64', href: rpm.url },
      { label: 'RPM ARM64', href: rpmArm64.url },
    ],
  },
  { id: 'nix', label: 'NIX', command: 'nix profile install github:WhoSowSee/mdv' },
  {
    id: 'source', label: 'SOURCE',
    command: 'git clone https://github.com/WhoSowSee/mdv.git\ncd mdv\ncargo install --path .',
  },
] as const satisfies readonly Omit<InstallMethod, 'note'>[];

type InstallMethodId = (typeof installCommands)[number]['id'];

export function localizeInstallMethods(notes: Record<InstallMethodId, string>, labels: Partial<Record<InstallMethodId, string>> = {}): InstallMethod[] {
  return installCommands.map((method) => ({
    ...method,
    label: labels[method.id] ?? method.label,
    note: notes[method.id],
  }));
}
