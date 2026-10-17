use std::path::PathBuf;

use gpui::{
    canvas, div, prelude::*, px, uniform_list, Context, ExternalPaths, Hsla, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render, SharedString, Subscription, Window,
};

use crate::compression::{self, registry::CompressorRegistry, CompressionError};
use crate::format::{InputFormat, OutputFormat};
use crate::image_item::{format_size, ImageItem, ProcessingStatus};
use crate::theme::Theme;

/// Width of the quality slider's track, in pixels.
const SLIDER_WIDTH: f32 = 120.0;

/// The root view for the Kompressor application.
pub struct Kompressor {
    items: Vec<ImageItem>,
    next_id: usize,
    output_format: OutputFormat,
    quality: u8,
    slider_dragging: bool,
    error_message: Option<String>,
    _appearance_observer: Subscription,
}

impl Kompressor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let weak = cx.weak_entity();
        let appearance_observer = window.observe_window_appearance(move |_window, cx| {
            weak.update(cx, |_, cx| cx.notify()).ok();
        });

        Self {
            items: Vec::new(),
            next_id: 0,
            output_format: OutputFormat::ALL[0],
            quality: 80,
            slider_dragging: false,
            error_message: None,
            _appearance_observer: appearance_observer,
        }
    }

    fn handle_dropped_paths(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        for path in paths {
            self.process_dropped_path(path.clone(), cx);
        }
        cx.notify();
    }

    /// Validates a single dropped file and, if it's a supported input image,
    /// adds it to the list and starts compressing it in the background.
    fn process_dropped_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        if InputFormat::from_path(&path).is_none() {
            self.error_message = Some(format!(
                "Unsupported file type for \"{file_name}\". Only JPG, JPEG and PNG are supported."
            ));
            return;
        }

        let original_size = match std::fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(err) => {
                self.error_message = Some(format!("Could not read \"{file_name}\": {err}"));
                return;
            }
        };

        let id = self.next_id;
        self.next_id += 1;
        // The output format and quality are captured now, so later changes to
        // the status bar's controls don't affect images already in the list.
        let output_format = self.output_format;
        let quality = self.quality;

        self.items.push(ImageItem {
            id,
            path: path.clone(),
            file_name: file_name.clone(),
            original_size,
            output_format,
            quality,
            status: ProcessingStatus::Processing,
        });

        let output_path = compression::output_path(&path, output_format);

        cx.spawn(async move |weak, cx| {
            let input = path;
            let output = output_path;
            let result: Result<u64, CompressionError> = cx
                .background_spawn(async move {
                    let registry = CompressorRegistry::new();
                    let compressor = registry.compressor_for(output_format);
                    compressor.compress(&input, &output, output_format, quality)?;
                    std::fs::metadata(&output)
                        .map(|metadata| metadata.len())
                        .map_err(|err| CompressionError(err.to_string()))
                })
                .await;

            weak.update(cx, |this, cx| {
                match result {
                    Ok(output_size) => {
                        if let Some(item) = this.items.iter_mut().find(|item| item.id == id) {
                            item.status = ProcessingStatus::Done { output_size };
                        }
                    }
                    Err(err) => {
                        if let Some(item) = this.items.iter_mut().find(|item| item.id == id) {
                            item.status = ProcessingStatus::Error(err.0.clone());
                        }
                        this.error_message = Some(format!("{file_name}: {}", err.0));
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Renders the scrollable list of images (or a placeholder when empty).
    /// This area, excluding the status bar, is the drop zone for new images.
    fn render_drop_zone(&mut self, theme: Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let header = div()
            .flex()
            .flex_row()
            .gap_1()
            .px_3()
            .py_1()
            .text_xs()
            .text_color(theme.text_muted)
            .border_b_1()
            .border_color(theme.border)
            .child(div().flex_1().child("Name"))
            .child(div().w(px(72.)).child("Original"))
            .child(div().w(px(80.)).child("Output"));

        if self.items.is_empty() {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .child(header)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.text_muted)
                        .text_sm()
                        .child("Drop images here to compress"),
                )
                .into_any_element();
        }

        let list = uniform_list(
            "image-list",
            self.items.len(),
            cx.processor(move |this, range: std::ops::Range<usize>, _window, _cx| {
                range
                    .map(|ix| render_row(&this.items[ix], theme, ix))
                    .collect::<Vec<_>>()
            }),
        )
        .flex_1();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(header)
            .child(list)
            .into_any_element()
    }

    /// Renders the non-scrollable status bar: output format radios on the
    /// left, quality slider on the right.
    fn render_status_bar(&mut self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .gap_3()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.surface)
            .child(self.render_format_radios(theme, cx))
            .child(self.render_quality_slider(theme, cx))
    }

    fn render_format_radios(&mut self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.output_format;
        div()
            .flex()
            .flex_row()
            .gap_3()
            .children(OutputFormat::ALL.into_iter().map(|format| {
                let is_selected = format == selected;
                div()
                    .id(format.label())
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.output_format = format;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .size(px(12.))
                            .rounded_full()
                            .border_1()
                            .border_color(if is_selected {
                                theme.accent
                            } else {
                                theme.border
                            })
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(is_selected, |dot| {
                                dot.child(div().size(px(6.)).rounded_full().bg(theme.accent))
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(if is_selected {
                                theme.text
                            } else {
                                theme.text_muted
                            })
                            .child(format.label()),
                    )
            }))
    }

    /// Renders the quality slider. Dragging anywhere on the track (even
    /// after the cursor leaves it) updates the quality, mirroring how a
    /// native slider behaves.
    fn render_quality_slider(&mut self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let quality = self.quality;
        let entity = cx.entity();
        let filled_width = px(SLIDER_WIDTH * quality as f32 / 100.0);

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                div()
                    .id("quality-slider")
                    .relative()
                    .w(px(SLIDER_WIDTH))
                    .h(px(16.))
                    .cursor_pointer()
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |event: &MouseDownEvent, _phase, _window, cx| {
                                        if event.button != MouseButton::Left
                                            || !bounds.contains(&event.position)
                                        {
                                            return;
                                        }
                                        let percent = ((event.position.x - bounds.origin.x)
                                            / bounds.size.width)
                                            .clamp(0., 1.);
                                        entity.update(cx, |this, cx| {
                                            this.quality = (percent * 100.0).round() as u8;
                                            this.slider_dragging = true;
                                            cx.notify();
                                        });
                                    }
                                });

                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |event: &MouseMoveEvent, _phase, _window, cx| {
                                        if !event.dragging() || !entity.read(cx).slider_dragging {
                                            return;
                                        }
                                        let percent = ((event.position.x - bounds.origin.x)
                                            / bounds.size.width)
                                            .clamp(0., 1.);
                                        entity.update(cx, |this, cx| {
                                            this.quality = (percent * 100.0).round() as u8;
                                            cx.notify();
                                        });
                                    }
                                });

                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |_event: &MouseUpEvent, _phase, _window, cx| {
                                        entity.update(cx, |this, _cx| {
                                            this.slider_dragging = false;
                                        });
                                    }
                                });
                            },
                        )
                        .size_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(6.))
                            .left_0()
                            .w_full()
                            .h(px(4.))
                            .rounded_full()
                            .bg(theme.border)
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .h_full()
                                    .w(filled_width)
                                    .rounded_full()
                                    .bg(theme.accent),
                            ),
                    ),
            )
            .child(
                div()
                    .w(px(32.))
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(format!("{quality}%")),
            )
    }
}

impl Render for Kompressor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::for_appearance(window.appearance());
        let error_message = self.error_message.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.text)
            .text_sm()
            .relative()
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_dropped_paths(paths.paths(), cx);
            }))
            .drag_over::<ExternalPaths>(move |style, _, _, _| {
                style.bg(theme.background.opacity(0.8))
            })
            .child(self.render_drop_zone(theme, cx))
            .child(self.render_status_bar(theme, cx))
            .when_some(error_message, |parent, message| {
                parent.child(render_error_popup(theme, message, cx))
            })
    }
}

/// Renders a single row in the image table: file name, original size, and
/// output size (or its current processing status).
fn render_row(item: &ImageItem, theme: Theme, ix: usize) -> impl IntoElement {
    let (output_text, output_color): (SharedString, Hsla) = match &item.status {
        ProcessingStatus::Processing => ("Processing…".into(), theme.text_muted),
        ProcessingStatus::Done { output_size } => (format_size(*output_size).into(), theme.text),
        ProcessingStatus::Error(_) => ("Error".into(), theme.error),
    };

    div()
        .id(("image-row", ix))
        .flex()
        .flex_row()
        .gap_1()
        .w_full()
        .px_3()
        .py_1()
        .bg(if ix.is_multiple_of(2) {
            theme.background
        } else {
            theme.surface
        })
        .child(
            div()
                .flex_1()
                .truncate()
                .whitespace_nowrap()
                .child(SharedString::from(item.file_name.clone())),
        )
        .child(div().w(px(72.)).child(format_size(item.original_size)))
        .child(div().w(px(80.)).text_color(output_color).child(output_text))
}

/// Renders a modal popup that overlays the whole window to show an error
/// message, dismissed with the "OK" button.
fn render_error_popup(
    theme: Theme,
    message: String,
    cx: &mut Context<Kompressor>,
) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(gpui::black().opacity(0.4))
        .child(
            div()
                .id("error-popup")
                .w(px(340.))
                .bg(theme.surface)
                .text_color(theme.text)
                .border_1()
                .border_color(theme.border)
                .rounded_md()
                .shadow_lg()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("Error"),
                )
                .child(div().text_xs().child(message))
                .child(
                    div().flex().justify_end().child(
                        div()
                            .id("error-ok")
                            .px_3()
                            .py_1()
                            .rounded_sm()
                            .bg(theme.accent)
                            .text_color(gpui::white())
                            .text_xs()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.error_message = None;
                                cx.notify();
                            }))
                            .child("OK"),
                    ),
                ),
        )
}
