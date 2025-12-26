use eldritch_core::Span;
use lsp_types::{Position, Range};

pub fn span_to_range(span: Span, source: &str) -> Range {
    let start_line_idx = span.line.saturating_sub(1);
    let mut current_line = 0;
    let mut offset = 0;
    let mut line_start_offset = 0;

    for (i, b) in source.bytes().enumerate() {
        if current_line == start_line_idx {
            line_start_offset = offset;
            break;
        }
        if b == b'\n' {
            current_line += 1;
            offset = i + 1;
        }
    }
    if current_line < start_line_idx {
        line_start_offset = offset;
    }

    let start_col = span.start.saturating_sub(line_start_offset);
    let (end_line, end_col) = byte_offset_to_pos(span.end, source);

    Range::new(
        Position::new(start_line_idx as u32, start_col as u32),
        Position::new(end_line as u32, end_col as u32),
    )
}

fn byte_offset_to_pos(offset: usize, source: &str) -> (usize, usize) {
    let mut line = 0;
    let mut last_line_start = 0;
    for (i, b) in source.bytes().enumerate() {
        if i == offset {
            return (line, i - last_line_start);
        }
        if b == b'\n' {
            line += 1;
            last_line_start = i + 1;
        }
    }
    (line, offset.saturating_sub(last_line_start))
}
