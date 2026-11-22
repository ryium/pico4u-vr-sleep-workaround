//! Scrollable activity & debug log viewer sheet.

use crate::ui::PicoSleepApp;
use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    div, prelude::*, px, rgb,
};

impl PicoSleepApp {
    pub(crate) fn render_logs_sheet(&self) -> AnyElement {
        let empty_placeholder = div()
            .text_size(px(12.0))
            .italic()
            .text_color(rgb(0x6b7280))
            .child("No logs available.");

        let log_rows = self.logs.iter().map(|entry| {
            div()
                .text_size(px(11.5))
                .line_height(px(17.0))
                .text_color(rgb(0xe5e7eb))
                .child(entry.clone())
        });

        div()
            .id("logs-scroll-box")
            .w_full()
            .h(px(240.0))
            .p_4()
            .rounded_lg()
            .bg(rgb(0x111827))
            .border_1()
            .border_color(rgb(0x1f2937))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_1()
            .when(self.logs.is_empty(), |el| el.child(empty_placeholder))
            .children(log_rows)
            .into_any_element()
    }
}
