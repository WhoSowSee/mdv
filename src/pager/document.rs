use super::*;
use crate::terminal::OutputStyle;
mod layout;
mod views;
pub(in crate::pager) use views::PagerLineNumberViews;

pub(crate) struct PagerDocument {
    warmup: Option<super::warmup::WarmupHandle>,
    reflow: Option<super::rendering::Reflow>,
    layout_width: Option<usize>,
    width_limit: Option<usize>,
    cached_layouts: std::collections::VecDeque<(usize, PagerContent)>,
    content: PagerContent,
    pub(in crate::pager) title: Option<String>,
    status_bar_transparent: bool,
    output_style: OutputStyle,
}

pub(in crate::pager) enum PagerContent {
    Static(String),
    LineNumbers(PagerLineNumberViews),
}

pub(in crate::pager) struct PagerDisplay {
    output: String,
    source_lines: Vec<Option<usize>>,
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

impl PagerContent {
    pub(in crate::pager) fn output(&self) -> Result<&str> {
        match self {
            Self::Static(output) => Ok(output),
            Self::LineNumbers(views) => Ok(&views.current()?.output),
        }
    }

    fn into_output(self) -> Result<String> {
        match self {
            Self::Static(output) => Ok(output),
            Self::LineNumbers(views) => Ok(views.current()?.output.clone()),
        }
    }
}

impl PagerDocument {
    pub(crate) fn new(output: String, output_style: OutputStyle) -> Self {
        Self::from_content(PagerContent::Static(output), output_style)
    }

    pub(in crate::pager) fn from_content(content: PagerContent, output_style: OutputStyle) -> Self {
        Self {
            content,
            warmup: None,
            reflow: None,
            layout_width: None,
            width_limit: None,
            cached_layouts: std::collections::VecDeque::new(),
            title: None,
            status_bar_transparent: false,
            output_style,
        }
    }

    pub(crate) fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub(super) fn with_reflow(
        mut self,
        reflow: Option<super::rendering::Reflow>,
        width: Option<usize>,
        limit: Option<usize>,
    ) -> Self {
        self.reflow = reflow;
        self.layout_width = width;
        self.width_limit = limit;
        self
    }

    pub(super) fn can_reflow(&self) -> bool {
        self.reflow.is_some()
    }

    pub(super) fn has_line_navigation(&self) -> Result<bool> {
        match &self.content {
            PagerContent::Static(_) => Ok(false),
            PagerContent::LineNumbers(views) => {
                Ok(views.current()?.source_lines.iter().any(Option::is_some))
            }
        }
    }

    pub(crate) const fn with_status_bar_transparent(mut self, transparent: bool) -> Self {
        self.status_bar_transparent = transparent;
        self
    }

    pub(crate) const fn status_bar_transparent(&self) -> bool {
        self.status_bar_transparent
    }

    pub(crate) const fn output_style(&self) -> OutputStyle {
        self.output_style
    }

    pub(in crate::pager) fn display_snapshot(&self) -> Result<(String, Option<LineNavigation>)> {
        match &self.content {
            PagerContent::Static(output) => Ok((output.clone(), None)),
            PagerContent::LineNumbers(views) => {
                let snapshot = views.snapshot()?;
                if let Some(warmup) = &self.warmup {
                    warmup.schedule(views.clone());
                }
                Ok(snapshot)
            }
        }
    }

    pub(super) fn attach_warmup(&mut self, warmup: super::warmup::WarmupHandle) {
        self.warmup = Some(warmup);
    }

    pub(super) fn inherit_warmup_from(&mut self, previous: &Self) {
        self.warmup = previous.warmup.clone();
        if let Some(warmup) = &self.warmup {
            warmup.reset();
        }
    }

    pub(in crate::pager) fn prepare_current_view(&self) -> Result<()> {
        self.content.output().map(|_| ())
    }

    pub(in crate::pager) fn into_output(self) -> Result<String> {
        self.content.into_output()
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

    pub(in crate::pager) fn cycle_line_number_mode(&mut self) -> Result<bool> {
        let PagerContent::LineNumbers(views) = &mut self.content else {
            return Ok(false);
        };
        views.cycle()?;
        Ok(true)
    }
}

pub(crate) type RefreshCallback = Arc<dyn Fn() -> Result<PagerDocument> + Send + Sync>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum PagerScreen {
    Alternate,
    InPlace,
}
