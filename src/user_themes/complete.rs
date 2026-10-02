use super::schema::{SyntaxFile, ThemeFile};
use super::*;

impl SyntaxFile {
    fn into_complete(self, name: &str) -> Result<SyntaxTheme> {
        Ok(SyntaxTheme {
            keyword: required(self.keyword, name, "syntax:keyword")?,
            string: required(self.string, name, "syntax:string")?,
            comment: required(self.comment, name, "syntax:comment")?,
            number: required(self.number, name, "syntax:number")?,
            operator: required(self.operator, name, "syntax:operator")?,
            function: required(self.function, name, "syntax:function")?,
            variable: required(self.variable, name, "syntax:variable")?,
            type_name: required(self.type_name, name, "syntax:type_name")?,
        })
    }
}

impl ThemeFile {
    pub(super) fn into_complete(self) -> Result<Theme> {
        if self.name.trim().is_empty() {
            bail!("Theme file is missing 'name' field");
        }
        if self.extends.is_some() {
            bail!("Embedded themes cannot use 'extends'");
        }
        let name = self.name.trim().to_string();
        let mut inline_style = InlineStyleSet::default();
        inline_style.apply_overrides(&self.inline_style);
        Ok(Theme {
            description: self
                .description
                .filter(|value| !value.trim().is_empty())
                .with_context(|| format!("Embedded theme '{name}' is missing 'description'"))?,
            pager: self
                .pager
                .with_context(|| format!("Embedded theme '{name}' is missing 'pager'"))?
                .into_complete(&name)?,
            text: required(self.text, &name, "text")?,
            text_light: required(self.text_light, &name, "text_light")?,
            h1: required(self.h1, &name, "h1")?,
            h2: required(self.h2, &name, "h2")?,
            h3: required(self.h3, &name, "h3")?,
            h4: required(self.h4, &name, "h4")?,
            h5: required(self.h5, &name, "h5")?,
            h6: required(self.h6, &name, "h6")?,
            code: self
                .code
                .with_context(|| format!("Embedded theme '{name}' is missing 'code'"))?
                .into_complete(&name, "code")?,
            quote: required(self.quote, &name, "quote")?,
            link: required(self.link, &name, "link")?,
            emphasis: self
                .emphasis
                .with_context(|| format!("Embedded theme '{name}' is missing 'emphasis'"))?
                .into_complete(&name, "emphasis")?,
            strong: self
                .strong
                .with_context(|| format!("Embedded theme '{name}' is missing 'strong'"))?
                .into_complete(&name, "strong")?,
            strikethrough: self
                .strikethrough
                .with_context(|| format!("Embedded theme '{name}' is missing 'strikethrough'"))?
                .into_complete(&name, "strikethrough")?,
            error: required(self.error, &name, "error")?,
            warning: required(self.warning, &name, "warning")?,
            strong_emphasis: self.strong_emphasis.unwrap_or_default().into_optional(),
            highlight: self
                .highlight
                .with_context(|| format!("Embedded theme '{name}' is missing 'highlight'"))?
                .into_complete(&name)?,
            background: self.background.resolve(&None),
            horizontal_rule: self.horizontal_rule.resolve(&None),
            footnote_separator: self.footnote_separator.resolve(&None),
            line_number: self
                .line_number
                .with_context(|| format!("Embedded theme '{name}' is missing 'line_number'"))?
                .into_complete(&name)?,
            table: self
                .table
                .with_context(|| format!("Embedded theme '{name}' is missing 'table'"))?
                .into_complete(&name)?,
            math: self
                .math
                .with_context(|| format!("Embedded theme '{name}' is missing 'math'"))?
                .into_complete(&name)?,
            front_matter: self
                .front_matter
                .with_context(|| format!("Embedded theme '{name}' is missing 'front_matter'"))?
                .into_complete(&name)?,
            code_block: self
                .code_block
                .with_context(|| format!("Embedded theme '{name}' is missing 'code_block'"))?
                .into_complete(&name)?,
            list: self
                .list
                .with_context(|| format!("Embedded theme '{name}' is missing 'list'"))?
                .into_complete(&name)?,
            todo: self
                .todo
                .with_context(|| format!("Embedded theme '{name}' is missing 'todo'"))?
                .into_complete(&name)?,
            details_border: self.details_border.resolve(&None),
            callout: self
                .callout
                .with_context(|| format!("Embedded theme '{name}' is missing 'callout'"))?
                .into_complete(&name)?,
            inline_style,
            syntax: self
                .syntax
                .with_context(|| format!("Embedded theme '{name}' is missing 'syntax'"))?
                .into_complete(&name)?,
            name,
            color_priorities: crate::theme::ColorPriorities::default(),
        })
    }
}

impl LineNumberFile {
    pub(super) fn into_complete(self, name: &str) -> Result<LineNumberTheme> {
        Ok(LineNumberTheme {
            number: required(self.number, name, "line_number:number")?,
            separator: required(self.separator, name, "line_number:separator")?,
        })
    }
}

impl TableFile {
    pub(super) fn into_complete(self, name: &str) -> Result<TableTheme> {
        Ok(TableTheme {
            header: required(self.header, name, "table:header")?,
            border: self.border.resolve(&None),
        })
    }
}

impl MathFile {
    pub(super) fn into_complete(self, name: &str) -> Result<MathTheme> {
        Ok(MathTheme {
            text: required(self.text, name, "math:text")?,
            border: self.border.resolve(&None),
        })
    }
}

impl FrontMatterFile {
    pub(super) fn into_complete(self, name: &str) -> Result<FrontMatterTheme> {
        Ok(FrontMatterTheme {
            title: required(self.title, name, "front_matter:title")?,
            key: required(self.key, name, "front_matter:key")?,
            value: required(self.value, name, "front_matter:value")?,
            border: self.border.resolve(&None),
        })
    }
}

impl CodeBlockFile {
    pub(super) fn into_complete(self, name: &str) -> Result<CodeBlockTheme> {
        Ok(CodeBlockTheme {
            label: required(self.label, name, "code_block:label")?,
            border: self.border.resolve(&None),
        })
    }
}

impl ListFile {
    pub(super) fn into_complete(self, name: &str) -> Result<ListTheme> {
        Ok(ListTheme {
            ordered: required(self.ordered, name, "list:ordered")?,
            unordered: required(self.unordered, name, "list:unordered")?,
        })
    }
}

impl TodoFile {
    pub(super) fn into_complete(self, name: &str) -> Result<TodoTheme> {
        Ok(TodoTheme {
            checked: required(self.checked, name, "todo:checked")?,
            unchecked: required(self.unchecked, name, "todo:unchecked")?,
        })
    }
}

impl CalloutPaletteFile {
    pub(super) fn into_complete(self, name: &str) -> Result<CalloutPalette> {
        Ok(CalloutPalette {
            note: required(self.note, name, "callout:palette:note")?,
            abstract_color: required(self.abstract_color, name, "callout:palette:abstract")?,
            info: required(self.info, name, "callout:palette:info")?,
            todo: required(self.todo, name, "callout:palette:todo")?,
            tip: required(self.tip, name, "callout:palette:tip")?,
            success: required(self.success, name, "callout:palette:success")?,
            question: required(self.question, name, "callout:palette:question")?,
            warning: required(self.warning, name, "callout:palette:warning")?,
            failure: required(self.failure, name, "callout:palette:failure")?,
            danger: required(self.danger, name, "callout:palette:danger")?,
            bug: required(self.bug, name, "callout:palette:bug")?,
            example: required(self.example, name, "callout:palette:example")?,
            quote: required(self.quote, name, "callout:palette:quote")?,
        })
    }
}

impl CalloutFile {
    pub(super) fn into_complete(self, name: &str) -> Result<CalloutTheme> {
        Ok(CalloutTheme {
            label: self.label.resolve(&None),
            border: self.border.resolve(&None),
            palette: self
                .palette
                .with_context(|| format!("Embedded theme '{name}' is missing 'callout:palette'"))?
                .into_complete(name)?,
        })
    }
}
