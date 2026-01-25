use eldritch::repl::Input;

/// A robust VT100/ANSI input parser that logs incoming bytes and swallows unknown sequences.
pub struct InputParser {
    pub buffer: Vec<u8>,
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}

impl InputParser {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn parse(&mut self, data: &[u8]) -> Vec<Input> {
        #[cfg(debug_assertions)]
        log::debug!("Received raw bytes: {data:02x?}");

        self.buffer.extend_from_slice(data);
        let mut inputs = Vec::new();

        // Process buffer
        loop {
            if self.buffer.is_empty() {
                break;
            }

            let b = self.buffer[0];

            if b == 0x1b {
                // Potential Escape Sequence
                // We need at least 2 bytes to decide type, or just 1 byte if it's strictly just ESC (unlikely in streams)
                // But we must handle split packets.
                if self.buffer.len() < 2 {
                    // Incomplete, wait for more data
                    break;
                }

                let second = self.buffer[1];
                match second {
                    b'[' => {
                        // CSI Sequence: ESC [ params final
                        // Params: 0x30-0x3F, Intermediate: 0x20-0x2F, Final: 0x40-0x7E
                        let mut end_idx = None;
                        for (i, &byte) in self.buffer.iter().enumerate().skip(2) {
                            if (0x40..=0x7E).contains(&byte) {
                                end_idx = Some(i);
                                break;
                            }
                        }

                        if let Some(end) = end_idx {
                            // We have a complete sequence
                            let seq = &self.buffer[0..=end];
                            if let Some(input) = self.parse_csi(seq) {
                                inputs.push(input);
                            } else {
                                #[cfg(debug_assertions)]
                                log::warn!("Ignored CSI sequence: {seq:02x?}");
                            }
                            // Consume
                            self.buffer.drain(0..=end);
                        } else {
                            // Incomplete CSI or very long garbage
                            if self.buffer.len() > 32 {
                                // Safety valve: sequence too long, probably garbage. Consume ESC and continue.
                                #[cfg(debug_assertions)]
                                log::warn!(
                                    "Dropping long incomplete CSI buffer: {:02x?}",
                                    &self.buffer[..32]
                                );
                                self.buffer.remove(0);
                            } else {
                                // Wait for more data
                                break;
                            }
                        }
                    }
                    b'O' => {
                        // SS3 Sequence: ESC O char
                        if self.buffer.len() < 3 {
                            break;
                        }
                        let code = self.buffer[2];
                        let _seq = &self.buffer[0..3];
                        if let Some(input) = self.parse_ss3(code) {
                            inputs.push(input);
                        } else {
                            #[cfg(debug_assertions)]
                            log::warn!("Ignored SS3 sequence: {_seq:02x?}");
                        }
                        self.buffer.drain(0..3);
                    }
                    _ => {
                        // Unknown Escape Sequence or Alt+Key
                        // To be safe and avoid "random characters injected", we consume ESC and the next char.
                        #[cfg(debug_assertions)]
                        log::warn!("Unknown Escape sequence start: 1b {second:02x}");
                        self.buffer.drain(0..2);
                    }
                }
            } else {
                // Regular character or Control Code
                match b {
                    b'\r' | b'\n' => inputs.push(Input::Enter),
                    0x7f | 0x08 => inputs.push(Input::Backspace),
                    0x03 => inputs.push(Input::Cancel), // Ctrl+C
                    0x04 => inputs.push(Input::EOF),    // Ctrl+D
                    0x0c => inputs.push(Input::ClearScreen), // Ctrl+L
                    0x09 => inputs.push(Input::Tab),
                    0x12 => inputs.push(Input::HistorySearch), // Ctrl+R
                    0x15 => inputs.push(Input::KillLine),      // Ctrl+U
                    0x0b => inputs.push(Input::KillToEnd),     // Ctrl+K
                    0x17 => inputs.push(Input::WordBackspace), // Ctrl+W
                    0x00 => inputs.push(Input::ForceComplete), // Ctrl+Space
                    0x01 => inputs.push(Input::Home),          // Ctrl+A
                    0x05 => inputs.push(Input::End),           // Ctrl+E
                    x if x >= 0x20 => inputs.push(Input::Char(x as char)),
                    _ => {
                        // Other control codes? Ignore them to prevent weirdness
                        #[cfg(debug_assertions)]
                        log::debug!("Ignored control char: {b:02x}");
                    }
                }
                self.buffer.remove(0);
            }
        }
        inputs
    }

    fn parse_csi(&self, seq: &[u8]) -> Option<Input> {
        // seq is like [0x1b, '[', ..., final]
        // Minimal length 3: \x1b[A
        if seq.len() < 3 {
            return None;
        }

        let final_byte = *seq.last()?;

        // Check for simple no-param sequences
        if seq.len() == 3 {
            return match final_byte {
                b'A' => Some(Input::Up),
                b'B' => Some(Input::Down),
                b'C' => Some(Input::Right),
                b'D' => Some(Input::Left),
                b'H' => Some(Input::Home),
                b'F' => Some(Input::End),
                _ => None,
            };
        }

        // Tilde sequences: \x1b[num~
        // e.g. \x1b[3~ (Del), \x1b[1~ (Home)
        if final_byte == b'~' {
            // Extract number between [ and ~
            let inner = &seq[2..seq.len() - 1];
            if let Ok(s) = std::str::from_utf8(inner) {
                return match s {
                    "1" | "7" => Some(Input::Home),
                    "4" | "8" => Some(Input::End),
                    "3" => Some(Input::Delete),
                    _ => None, // PageUp(5), PageDown(6), Insert(2) - ignore for now
                };
            }
        }

        None
    }

    fn parse_ss3(&self, code: u8) -> Option<Input> {
        match code {
            b'A' => Some(Input::Up),
            b'B' => Some(Input::Down),
            b'C' => Some(Input::Right),
            b'D' => Some(Input::Left),
            b'H' => Some(Input::Home),
            b'F' => Some(Input::End),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_parser_simple() {
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"hello");
        assert_eq!(inputs.len(), 5);
        assert_eq!(inputs[0], Input::Char('h'));
    }

    #[test]
    fn test_input_parser_csi_arrow() {
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"\x1b[A");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Up);
    }

    #[test]
    fn test_input_parser_ss3_arrow() {
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"\x1bOA");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Up);
    }

    #[test]
    fn test_input_parser_split_packet() {
        let mut parser = InputParser::new();
        // Packet 1: Partial CSI
        let inputs = parser.parse(b"\x1b[");
        assert_eq!(inputs.len(), 0);

        // Packet 2: Remainder
        let inputs = parser.parse(b"A");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Up);
    }

    #[test]
    fn test_input_parser_unknown_csi_swallowed() {
        let mut parser = InputParser::new();
        // Unknown CSI: \x1b[99;99X (Random Garbage)
        // Should produce NO inputs and NOT leak 'X'
        let inputs = parser.parse(b"\x1b[99;99X");
        assert_eq!(inputs.len(), 0);

        // Verify buffer is drained
        assert!(parser.buffer.is_empty());

        // Followed by valid input
        let inputs = parser.parse(b"A");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Char('A'));
    }

    #[test]
    fn test_input_parser_delete() {
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"\x1b[3~");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Delete);
    }

    #[test]
    fn test_input_parser_home_end_ctrl_chars() {
        let mut parser = InputParser::new();
        // Ctrl+A (Home)
        let inputs = parser.parse(b"\x01");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Home);

        // Ctrl+E (End)
        let inputs = parser.parse(b"\x05");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::End);
    }
}
