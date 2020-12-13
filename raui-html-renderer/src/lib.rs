use raui_core::prelude::*;
use std::{
    collections::HashMap,
    fmt::{Error, Write},
};

#[allow(unused_macros)]
macro_rules! table {
    () => (HashMap::new());
    { $( $key:expr => $value:expr ),* } => {
        {
            let mut result = HashMap::new();
            $(
                result.insert($key.to_string(), $value.to_string());
            )*
            result
        }
    };
}

#[allow(unused_macros)]
macro_rules! node {
    (
        $context:ident : $tag:ident
        [ $writer:expr ]
        $( level = { $level:expr } )?
        $( styles = { $( $style_key:expr => $style_value:expr ),* } )?
        $( attribs = { $( $attrib_key:expr => $attrib_value:expr ),* } )?
        ($writer_name:ident, $level_name:ident)
    ) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_assignments)]
        {
            let mut level = 0;
            $(
                level = $level;
            )?
            let mut styles = Styles::new();
            $(
                styles = table!{ $( $style_key => $style_value ),* };
            )?
            let mut attribs = Attribs::new();
            $(
                attribs = table!{ $( $attrib_key => $attrib_value ),* };
            )?
            $context.inline_node(
                &stringify!($tag),
                &styles,
                &attribs,
                $writer,
                level,
            )?;
        }
    };
    (
        $context:ident : $tag:ident
        [ $writer:expr ]
        $( level = { $level:expr } )?
        $( styles = { $( $style_key:expr => $style_value:expr ),* } )?
        $( attribs = { $( $attrib_key:expr => $attrib_value:expr ),* } )?
        $code:block
        ($writer_name:ident, $level_name:ident)
    ) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_assignments)]
        {
            let mut level = 0;
            $(
                level = $level;
            )?
            let mut styles = Styles::new();
            $(
                styles = table!{ $( $style_key => $style_value ),* };
            )?
            let mut attribs = Attribs::new();
            $(
                attribs = table!{ $( $attrib_key => $attrib_value ),* };
            )?
            $context.with_node(
                &stringify!($tag),
                &styles,
                &attribs,
                $writer,
                level,
                |$writer_name, $level_name| {
                    $code
                    Ok(())
                }
            )?;
        }
    };
}

type Styles = HashMap<String, String>;
type Attribs = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct HtmlRenderer {
    pub indent: usize,
    pub title: Option<String>,
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self {
            indent: 2,
            title: None,
        }
    }
}

impl Renderer<String, Error> for HtmlRenderer {
    fn render(&mut self, tree: &WidgetUnit, _layout: &Layout) -> Result<String, Error> {
        let mut result = String::new();
        self.write_document(&mut result, tree)?;
        Ok(result)
    }
}

#[allow(dead_code)]
impl HtmlRenderer {
    fn write_line<W>(&self, line: &str, writer: &mut W, level: usize) -> Result<(), Error>
    where
        W: Write,
    {
        for _ in 0..(level * self.indent) {
            writer.write_char(' ')?;
        }
        writer.write_str(line)?;
        writer.write_char('\n')?;
        Ok(())
    }

    fn inline_node<W>(
        &self,
        name: &str,
        styles: &Styles,
        attribs: &Attribs,
        writer: &mut W,
        level: usize,
    ) -> Result<(), Error>
    where
        W: Write,
    {
        let styles = Self::stringify_styles_attr(styles)?;
        let attribs = Self::stringify_attribs(attribs)?;
        self.write_line(
            &format!(r#"<{} {} {}/>"#, name, styles, attribs),
            writer,
            level,
        )
    }

    fn with_node<W, F>(
        &self,
        name: &str,
        styles: &Styles,
        attribs: &Attribs,
        writer: &mut W,
        level: usize,
        mut f: F,
    ) -> Result<(), Error>
    where
        W: Write,
        F: FnMut(&mut W, usize) -> Result<(), Error>,
    {
        self.start_node(name, styles, attribs, writer, level)?;
        f(writer, level + 1)?;
        self.stop_node(name, writer, level)
    }

    fn start_node<W>(
        &self,
        name: &str,
        styles: &Styles,
        attribs: &Attribs,
        writer: &mut W,
        level: usize,
    ) -> Result<(), Error>
    where
        W: Write,
    {
        let styles = Self::stringify_styles_attr(styles)?;
        let attribs = Self::stringify_attribs(attribs)?;
        self.write_line(
            &format!(r#"<{} {} {}>"#, name, styles, attribs),
            writer,
            level,
        )
    }

    fn stop_node<W>(&self, name: &str, writer: &mut W, level: usize) -> Result<(), Error>
    where
        W: Write,
    {
        self.write_line(&format!(r#"</{}>"#, name), writer, level)
    }

    fn stringify_styles_attr(styles: &Styles) -> Result<String, Error> {
        let mut result = String::new();
        if !styles.is_empty() {
            result.write_str(r#"style=""#)?;
            for (key, value) in styles {
                result.write_str(key)?;
                result.write_str(": ")?;
                result.write_str(value)?;
                result.write_char(';')?;
            }
            result.write_str(r#"""#)?;
        }
        Ok(result)
    }

    fn stringify_attribs(attribs: &Attribs) -> Result<String, Error> {
        Ok(attribs
            .iter()
            .map(|(k, v)| format!(r#"{}="{}""#, k, v))
            .collect::<Vec<String>>()
            .join(" "))
    }

    fn write_document<W>(&self, writer: &mut W, tree: &WidgetUnit) -> Result<(), Error>
    where
        W: Write,
    {
        self.write_line(r#"<!DOCTYPE html>"#, writer, 0)?;
        node!(self: html [writer] attribs={"lang" => "en", "dir" => "ltr"} {
            node!(self: head [writer] level={level} {
                self.write_line(r#"<meta charset="utf-8">"#, writer, level)?;
                if let Some(title) = &self.title {
                    self.write_line(&format!(r#"<title>{}</title>"#, title), writer, level)?;
                }
            } (writer, level));
            node!(self: body [writer] level={level} {
                self.write_node(writer, tree, level)?;
            } (writer, level));
        } (writer, level));
        Ok(())
    }

    fn write_node<W>(&self, writer: &mut W, tree: &WidgetUnit, level: usize) -> Result<(), Error>
    where
        W: Write,
    {
        self.write_node_with_styles_and_attribs(writer, tree, level, Styles::new(), Attribs::new())
    }

    fn write_node_with_styles_and_attribs<W>(
        &self,
        writer: &mut W,
        tree: &WidgetUnit,
        level: usize,
        _styles: Styles,
        _attribs: Attribs,
    ) -> Result<(), Error>
    where
        W: Write,
    {
        match tree {
            WidgetUnit::None => {}
            WidgetUnit::ContentBox(ContentBox { items, .. }) => {
                node!(self: div [writer] level={level} {
                    for item in items {
                        self.write_node(writer, &item.slot, level)?;
                    }
                } (writer, level));
            }
            WidgetUnit::FlexBox(FlexBox { items, .. }) => {
                node!(self: div [writer] level={level} {
                    for item in items {
                        self.write_node(writer, &item.slot, level)?;
                    }
                } (writer, level));
            }
            WidgetUnit::GridBox(GridBox { items, .. }) => {
                node!(self: div [writer] level={level} {
                    for item in items {
                        self.write_node(writer, &item.slot, level)?;
                    }
                } (writer, level));
            }
            WidgetUnit::SizeBox(SizeBox { slot, .. }) => {
                node!(self: div [writer] level={level} {
                    self.write_node(writer, slot, level)?;
                } (writer, level));
            }
            WidgetUnit::ImageBox(ImageBox { .. }) => {
                node!(self: div [writer] level={level} {
                } (writer, level));
            }
            WidgetUnit::TextBox(TextBox { text, .. }) => {
                node!(self: span [writer] level={level} {
                    self.write_line(text, writer, level)?;
                } (writer, level));
            }
        }
        Ok(())
    }
}
