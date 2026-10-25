mod view;

use std::path::PathBuf;

use gpui::{div, prelude::*, Context, ExternalPaths, FocusHandle, Render, Subscription, Window};

use crate::compression::{self, registry::CompressorRegistry, CompressionError};
use crate::format::{InputFormat, OutputFormat};
use crate::image_item::{ImageItem, ProcessingStatus};
use crate::theme::Theme;
use crate::{Close, Minimize};

/// The root view for the Kompressor application.
pub struct Kompressor {
    focus_handler: FocusHandle,
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
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window);

        Self {
            focus_handler: focus_handle,
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
}

impl Render for Kompressor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::for_appearance(window.appearance());
        let error_message = self.error_message.clone();

        div()
            .on_action(|_: &Close, window, _| window.remove_window())
            .on_action(|_: &Minimize, window, _| window.minimize_window())
            .track_focus(&self.focus_handler)
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
            .child(view::render_drop_zone(self, theme, cx))
            .child(view::render_status_bar(self, theme, cx))
            .when_some(error_message, |parent, message| {
                parent.child(view::render_error_popup(theme, message, cx))
            })
    }
}
