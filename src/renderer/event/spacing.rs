use super::{Event, EventRenderer, Tag, TagEnd};
use crate::block_spacing::BlockElement;

impl EventRenderer<'_> {
    pub(super) fn prepare_block_spacing_elements(&mut self, events: &[Event<'static>]) {
        let mut list_elements = Vec::new();
        let mut list_stack = Vec::new();
        let mut blockquote_elements = Vec::new();
        let mut blockquote_stack = Vec::new();

        for event in events {
            match event {
                Event::Start(Tag::List(start)) => {
                    let element = if start.is_some() {
                        BlockElement::OrderedList
                    } else {
                        BlockElement::UnorderedList
                    };
                    let index = list_elements.len();
                    list_elements.push(element);
                    list_stack.push(index);
                }
                Event::TaskListMarker(_) => {
                    if let Some(index) = list_stack.last().copied() {
                        list_elements[index] = BlockElement::TaskList;
                    }
                }
                Event::End(TagEnd::List(_)) => {
                    list_stack.pop();
                }
                Event::Start(Tag::BlockQuote(kind)) => {
                    let is_callout = kind.is_some();
                    let index = blockquote_elements.len();
                    blockquote_elements.push(if is_callout {
                        BlockElement::Callout
                    } else {
                        BlockElement::Blockquote
                    });
                    blockquote_stack.push((index, !is_callout));
                }
                Event::Text(text) => {
                    if let Some((index, marker_pending)) = blockquote_stack.last_mut()
                        && *marker_pending
                        && !text.trim().is_empty()
                    {
                        if Self::is_callout_marker_text(text) {
                            blockquote_elements[*index] = BlockElement::Callout;
                        }
                        *marker_pending = false;
                    }
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    blockquote_stack.pop();
                }
                _ => {}
            }
        }

        self.prepared_list_spacing_elements = list_elements.into();
        self.prepared_blockquote_spacing_elements = blockquote_elements.into();
    }
}
