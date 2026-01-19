use crate::runtime::BacktraceFrame;
use crate::vm::VM;
use std::io::Write;

impl<W: Write> VM<W> {
    pub fn capture_backtrace(&self) -> Vec<BacktraceFrame> {
        let mut frames = Vec::new();

        for frame in &self.frames {
            let file = frame
                .function
                .file_path
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            let line = frame.function.get_line_for_ip(frame.ip).unwrap_or(0);

            let function_name = frame.function.name.clone();

            let class = frame
                .called_class
                .clone()
                .or_else(|| frame.this.as_ref().map(|obj| obj.class_name.clone()));

            let type_ = if class.is_some() {
                Some("->".to_string())
            } else {
                None
            };

            frames.push(BacktraceFrame {
                file,
                line,
                function: function_name,
                class,
                type_,
            });
        }

        frames.reverse();
        frames
    }
}
