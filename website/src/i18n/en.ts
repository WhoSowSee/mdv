import { localizeInstallMethods, usageCommands } from '../data/site';

export const en = {
  meta: {
    title: 'mdv | Markdown, rendered right in your terminal',
    description: 'A fast, themeable terminal Markdown viewer written in Rust. Syntax highlighting, flexible layouts, rich links and live file monitoring.',
    skip: 'Skip to content',
  },
  nav: {
    label: 'Main navigation', home: 'mdv home', features: 'Features', docs: 'Documentation',
    docsShort: 'Docs', install: 'Installation', github: 'GitHub repository',
    language: 'Переключить на русский', pause: 'Pause animations', resume: 'Resume animations',
  },
  hero: {
    before: 'Markdown, ', accent: 'rendered', after: ' right in your terminal',
    lead: 'A fast, themeable viewer with syntax highlighting, flexible layout, rich links and live file monitoring.',
    start: 'Get Started', docs: 'View Docs',
  },
  sections: {
    usage: 'Usage', usageLead: 'Open a file, follow changes while you write, or read Markdown from another command.',
    features: 'Everything you need', featuresLead: 'Small tool. Plenty of possibilities.',
    faq: 'FAQ', faqLead: 'Quick answers.',
    finalFirst: 'Your Markdown.', finalLast: 'Your terminal.',
    finalLead: 'Install in one command and start reading Markdown in your terminal.',
  },
  showcase: {
    title: 'Made for reading', lead: 'Precise rendering inside ANSI-capable terminals, tuned to your workspace.',
    themes: 'Rich theming', themesText: 'Built-in themes, separate code palettes and on-the-fly overrides for UI and syntax colors. Add your own YAML theme and make it yours.',
    colors: 'Example colors from built-in themes', monitor: 'Live monitoring',
    monitorBefore: 'Edit in one pane, read in another. ', monitorAfter: ' refreshes the rendered view when the source file changes.',
    draft: '# A work in progress', draftText: 'Save a thought. Keep reading.', refresh: 'edit → save → refreshed view',
    layout: 'Layout under control', layoutText: 'Set width, wrapping, heading layout and table behavior. Choose clickable links, inline URLs or a reference table at the end.',
    settings: 'Example layout settings',
  },
  install: {
    title: 'Install', lead: 'Pick your flavor.', method: 'Installation method',
    copyLabel: 'Copy installation command', scrollLabel: 'Scroll installation command horizontally', binary: 'Prefer a ready-made binary?',
    download: 'Download a release ↗', copied: 'Copied to clipboard', failed: 'COPY FAILED. SELECT THE COMMAND TO COPY IT.',
  },
  terminal: {
    label: 'Illustration of Markdown rendered in a terminal', example: 'Rendered Markdown example',
    description: 'Fast, themeable Markdown viewer for the terminal',
    features: 'Features', rendering: 'Terminal-first rendering', themes: 'Rich theming:', monitoring: 'Live monitoring with',
  },
  footer: {
    egg: 'Discover the mdv easter egg',
    links: 'Project links', license: 'License', found: 'YOU FOUND THE DRIP!',
  },
  docs: {
    title: 'mdv documentation', navigation: 'Documentation navigation', browse: 'Browse documentation',
    search: 'Search documentation', placeholder: 'Search...', results: 'Search results', pages: 'Documentation pages',
    copyCode: 'Copy code',
    searching: 'Searching...', noResults: 'No matching pages. Try another term.',
    unavailable: 'Search unavailable. Edit your search to try again.', matches: 'Matching pages:',
    adjacent: 'Adjacent documentation pages', previous: 'Previous', next: 'Next', onPage: 'On this page',
    groups: { 'Getting started': 'Getting started', Reading: 'Reading', Formatting: 'Formatting', Customization: 'Customization', Reference: 'Reference' },
  },
  preview: {
    source: 'Markdown source', result: 'mdv output', command: 'Command', version: 'Version', keys: 'Keys',
  },
  usage: [
    { description: 'Render a Markdown file', command: usageCommands.file },
    { description: 'Refresh the view whenever the file changes', command: usageCommands.monitor },
    { description: 'Browse Markdown files in the current directory', command: usageCommands.browser },
    { description: 'Read Markdown from another command', command: usageCommands.stdin },
    { description: 'Apply a built-in preset', command: usageCommands.preset },
  ],
  features: [
    { title: 'Syntax highlighting', text: 'Highlighted code blocks with a separate palette, optional line numbers and language icons.' },
    { title: 'Reusable presets', text: 'Start with reader, compact, pretty or showcase. Save your own combination of settings in YAML.' },
    { title: 'Built-in pager', text: 'Search, select and copy text. Jump to a source line or open the file in your editor.' },
    { title: 'Document browser', text: 'Find Markdown files interactively. Browse a directory while respecting hidden files and Git ignore rules.' },
    { title: 'Start anywhere', text: 'Use --from to begin at a section, or --reverse to read the document from the end.' },
    { title: 'Script friendly', text: 'Read stdin, export HTML, control color output and share configuration across your machines.' },
  ],
  installMethods: localizeInstallMethods({
    cargo: 'Requires Rust 1.95 or newer and Cargo. Works on Windows, macOS and Linux.',
    windows: 'Windows x64 and ARM64. Downloads the latest release, verifies SHA-256 and adds mdv to your user PATH.',
    unix: 'Linux and macOS, x86_64 and ARM64. Requires curl and unzip. Installs to ~/.local/bin after SHA-256 verification.',
    arch: 'Downloads the package into the current directory, then installs it with pacman. Arch Linux x86_64; requires curl and glibc 2.39 or newer.',
    rpm: 'Downloads and installs the RPM for x86_64; for ARM64, replace x86_64 with aarch64 in both lines. Requires curl, dnf and glibc 2.39 or newer.',
    nix: 'Requires Nix with flakes enabled. Supports x86_64 and ARM64 Linux.',
    source: 'Requires Git and Rust 1.95 or newer. Builds and installs mdv from the repository.',
  }),
  questions: [
    { question: 'Which terminals work?', answer: 'Use a terminal with ANSI color support. A Nerd Font is needed only for features that display Nerd Font icons.' },
    { question: 'How do I change the theme?', answer: 'Pass --theme nord for a single run, or set theme in your configuration file. Create a configuration with mdv --init-config.' },
    { question: 'Can I read from stdin?', answer: 'Yes. Pipe Markdown with cat file.md | mdv, or use mdv - to read standard input explicitly.' },
    { question: 'Is mdv open source?', answer: 'Yes. mdv is written in Rust, released under the MIT license and developed on GitHub.' },
  ],
};
