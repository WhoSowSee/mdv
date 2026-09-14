use super::*;

pub(crate) struct PagerDocument {
    content: PagerContent,
    pub(in crate::pager) source: String,
    pub(in crate::pager) title: Option<String>,
    status_bar_transparent: bool,
}

pub(in crate::pager) enum PagerContent {
    Static(String),
    LineNumbers(PagerLineNumberViews),
}

pub(in crate::pager) struct PagerDisplay {
    output: String,
    source_lines: Vec<Option<usize>>,
}

pub(in crate::pager) struct PagerLineNumberViews {
    mode: PagerLineNumberMode,
    unnumbered: PagerDisplay,
    rendered: PagerDisplay,
    source: PagerDisplay,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::pager) enum PagerLineNumberMode {
    Off,
    Rendered,
    Source,
}

impl PagerLineNumberMode {
    const fn next(self) -> Self {
        match self {
            Self::Off => Self::Rendered,
            Self::Rendered => Self::Source,
            Self::Source => Self::Off,
        }
    }
}

impl PagerDisplay {
    pub(in crate::pager) fn new(output: String, source_lines: Vec<Option<usize>>) -> Self {
        Self {
            output,
            source_lines,
        }
    }
}

impl PagerLineNumberViews {
    pub(in crate::pager) fn new(
        mode: PagerLineNumberMode,
        unnumbered: PagerDisplay,
        rendered: PagerDisplay,
        source: PagerDisplay,
    ) -> Self {
        Self {
            mode,
            unnumbered,
            rendered,
            source,
        }
    }

    fn current(&self) -> &PagerDisplay {
        match self.mode {
            PagerLineNumberMode::Off => &self.unnumbered,
            PagerLineNumberMode::Rendered => &self.rendered,
            PagerLineNumberMode::Source => &self.source,
        }
    }

    fn cycle(&mut self) {
        self.mode = self.mode.next();
    }

    fn snapshot(&self) -> (String, Option<LineNavigation>) {
        let current = self.current();
        let navigation = current.source_lines.iter().any(Option::is_some).then(|| {
            if self.mode == PagerLineNumberMode::Source {
                LineNavigation::from_current(current.source_lines.clone())
            } else {
                LineNavigation::new(
                    self.source.output.clone(),
                    current.source_lines.clone(),
                    self.source.source_lines.clone(),
                )
            }
        });
        (current.output.clone(), navigation)
    }
}

impl PagerContent {
    pub(in crate::pager) fn output(&self) -> &str {
        match self {
            Self::Static(output) => output,
            Self::LineNumbers(views) => &views.current().output,
        }
    }
}

impl PagerDocument {
    pub(crate) fn new(output: String, source: String) -> Self {
        Self::from_content(PagerContent::Static(output), source)
    }

    pub(in crate::pager) fn from_content(content: PagerContent, source: String) -> Self {
        Self {
            content,
            source,
            title: None,
            status_bar_transparent: false,
        }
    }

    pub(crate) fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub(crate) const fn with_status_bar_transparent(mut self, transparent: bool) -> Self {
        self.status_bar_transparent = transparent;
        self
    }

    pub(crate) const fn status_bar_transparent(&self) -> bool {
        self.status_bar_transparent
    }

    pub(in crate::pager) fn display_snapshot(&self) -> (String, Option<LineNavigation>) {
        match &self.content {
            PagerContent::Static(output) => (output.clone(), None),
            PagerContent::LineNumbers(views) => views.snapshot(),
        }
    }

    pub(in crate::pager) fn line_number_mode(&self) -> Option<PagerLineNumberMode> {
        match &self.content {
            PagerContent::Static(_) => None,
            PagerContent::LineNumbers(views) => Some(views.mode),
        }
    }

    pub(in crate::pager) fn preserve_line_number_mode_from(&mut self, previous: &Self) {
        if let (Some(mode), PagerContent::LineNumbers(views)) =
            (previous.line_number_mode(), &mut self.content)
        {
            views.mode = mode;
        }
    }

    pub(in crate::pager) fn cycle_line_number_mode(
        &mut self,
    ) -> Option<(String, Option<LineNavigation>)> {
        let PagerContent::LineNumbers(views) = &mut self.content else {
            return None;
        };
        views.cycle();
        Some(views.snapshot())
    }
}

pub(crate) type RefreshCallback = Arc<dyn Fn() -> Result<PagerDocument> + Send + Sync>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum PagerScreen {
    Alternate,
    InPlace,
}
