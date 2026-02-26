use std::io;
use std::sync::Arc;

use wezterm_term::color::{ColorAttribute, ColorPalette};
use wezterm_term::{
  Blink, CellAttributes, Intensity, Terminal, TerminalConfiguration, TerminalSize, Underline,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct EmulatorConfig {
  pub(crate) rows: u16,
  pub(crate) cols: u16,
  pub(crate) scrollback_limit: usize,
}

pub(crate) trait TerminalEmulator: Send {
  fn apply_output(&mut self, bytes: &[u8]);
  fn set_size(&mut self, rows: u16, cols: u16);
  fn cursor_position(&self) -> (u16, u16);
  fn snapshot_lines(&self) -> Vec<String>;
  fn snapshot_ansi(&self) -> Vec<u8>;
}

#[derive(Debug)]
struct WeztermConfig {
  scrollback_limit: usize,
}

impl TerminalConfiguration for WeztermConfig {
  fn scrollback_size(&self) -> usize {
    self.scrollback_limit
  }

  fn color_palette(&self) -> ColorPalette {
    ColorPalette::default()
  }
}

pub(crate) struct WeztermEmulator {
  terminal: Terminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AttrState {
  intensity: Intensity,
  underline: Underline,
  blink: Blink,
  italic: bool,
  reverse: bool,
  strikethrough: bool,
  invisible: bool,
  overline: bool,
  fg: ColorAttribute,
  bg: ColorAttribute,
}

impl Default for AttrState {
  fn default() -> Self {
    Self {
      intensity: Intensity::Normal,
      underline: Underline::None,
      blink: Blink::None,
      italic: false,
      reverse: false,
      strikethrough: false,
      invisible: false,
      overline: false,
      fg: ColorAttribute::Default,
      bg: ColorAttribute::Default,
    }
  }
}

impl AttrState {
  fn from_attrs(attrs: &CellAttributes) -> Self {
    Self {
      intensity: attrs.intensity(),
      underline: attrs.underline(),
      blink: attrs.blink(),
      italic: attrs.italic(),
      reverse: attrs.reverse(),
      strikethrough: attrs.strikethrough(),
      invisible: attrs.invisible(),
      overline: attrs.overline(),
      fg: attrs.foreground(),
      bg: attrs.background(),
    }
  }
}

pub(crate) fn create_emulator(config: EmulatorConfig) -> Box<dyn TerminalEmulator> {
  create_emulator_with_writer(config, None)
}

pub(crate) fn create_emulator_with_writer(
  config: EmulatorConfig,
  writer: Option<Box<dyn io::Write + Send>>,
) -> Box<dyn TerminalEmulator> {
  Box::new(WeztermEmulator::new(config, writer))
}

impl WeztermEmulator {
  pub(crate) fn new(config: EmulatorConfig, writer: Option<Box<dyn io::Write + Send>>) -> Self {
    let size = TerminalSize {
      rows: config.rows as usize,
      cols: config.cols as usize,
      pixel_width: 0,
      pixel_height: 0,
      dpi: 0,
    };
    let config = Arc::new(WeztermConfig {
      scrollback_limit: config.scrollback_limit,
    });
    let writer = writer.unwrap_or_else(|| Box::new(io::sink()));
    let terminal = Terminal::new(size, config, "golutra", "1.0", writer);
    Self { terminal }
  }
}

impl TerminalEmulator for WeztermEmulator {
  fn apply_output(&mut self, bytes: &[u8]) {
    self.terminal.advance_bytes(bytes);
  }

  fn set_size(&mut self, rows: u16, cols: u16) {
    let size = TerminalSize {
      rows: rows as usize,
      cols: cols as usize,
      pixel_width: 0,
      pixel_height: 0,
      dpi: 0,
    };
    self.terminal.resize(size);
  }

  fn cursor_position(&self) -> (u16, u16) {
    let pos = self.terminal.cursor_pos();
    let row = if pos.y <= 0 {
      0
    } else {
      (pos.y as u64).min(u16::MAX as u64) as u16
    };
    let col = (pos.x).min(u16::MAX as usize) as u16;
    (row, col)
  }

  fn snapshot_lines(&self) -> Vec<String> {
    let screen = self.terminal.screen();
    let visible_rows = screen.physical_rows.max(1);
    let start = screen.phys_row(0);
    let end = start.saturating_add(visible_rows);
    let mut lines: Vec<String> = screen
      .lines_in_phys_range(start..end)
      .into_iter()
      .map(|line| line.as_str().trim_end().to_string())
      .collect();
    while lines.len() < visible_rows {
      lines.push(String::new());
    }
    lines
  }

  fn snapshot_ansi(&self) -> Vec<u8> {
    let pos = self.terminal.cursor_pos();
    let cursor_row = if pos.y <= 0 {
      0
    } else {
      (pos.y as u64).min(u16::MAX as u64) as u16
    };
    let cursor_col = (pos.x as u64).min(u16::MAX as u64) as u16;
    serialize_screen_to_ansi(self.terminal.screen(), cursor_row, cursor_col)
  }
}

fn emit_sgr(output: &mut String, params: &[String]) {
  if params.is_empty() {
    return;
  }
  output.push_str("\x1b[");
  output.push_str(&params.join(";"));
  output.push('m');
}

fn push_color_params(params: &mut Vec<String>, color: ColorAttribute, is_fg: bool) {
  match color {
    ColorAttribute::Default => {
      params.push((if is_fg { 39 } else { 49 }).to_string());
    }
    ColorAttribute::PaletteIndex(idx) => {
      let idx = idx as u16;
      if idx < 8 {
        let base = if is_fg { 30 } else { 40 };
        params.push((base + idx).to_string());
      } else if idx < 16 {
        let base = if is_fg { 90 } else { 100 };
        params.push((base + (idx - 8)).to_string());
      } else {
        let base = if is_fg { 38 } else { 48 };
        params.push(format!("{base};5;{idx}"));
      }
    }
    ColorAttribute::TrueColorWithPaletteFallback(color, _)
    | ColorAttribute::TrueColorWithDefaultFallback(color) => {
      let (r, g, b, _) = color.as_rgba_u8();
      let base = if is_fg { 38 } else { 48 };
      params.push(format!("{base};2;{r};{g};{b}"));
    }
  }
}

fn emit_attr_delta(output: &mut String, current: &mut AttrState, next: AttrState) {
  if *current == next {
    return;
  }
  let mut params = Vec::new();
  if current.intensity != next.intensity {
    let code = match next.intensity {
      Intensity::Normal => 22,
      Intensity::Bold => 1,
      Intensity::Half => 2,
    };
    params.push(code.to_string());
  }
  if current.italic != next.italic {
    params.push(if next.italic { "3" } else { "23" }.to_string());
  }
  if current.underline != next.underline {
    let code = match next.underline {
      Underline::None => "24".to_string(),
      Underline::Single => "4".to_string(),
      Underline::Double => "4:2".to_string(),
      Underline::Curly => "4:3".to_string(),
      Underline::Dotted => "4:4".to_string(),
      Underline::Dashed => "4:5".to_string(),
    };
    params.push(code);
  }
  if current.blink != next.blink {
    let code = match next.blink {
      Blink::None => 25,
      Blink::Slow => 5,
      Blink::Rapid => 6,
    };
    params.push(code.to_string());
  }
  if current.reverse != next.reverse {
    params.push(if next.reverse { "7" } else { "27" }.to_string());
  }
  if current.strikethrough != next.strikethrough {
    params.push(if next.strikethrough { "9" } else { "29" }.to_string());
  }
  if current.invisible != next.invisible {
    params.push(if next.invisible { "8" } else { "28" }.to_string());
  }
  if current.overline != next.overline {
    params.push(if next.overline { "53" } else { "55" }.to_string());
  }
  if current.fg != next.fg {
    push_color_params(&mut params, next.fg, true);
  }
  if current.bg != next.bg {
    push_color_params(&mut params, next.bg, false);
  }
  emit_sgr(output, &params);
  *current = next;
}

fn serialize_line_to_ansi(
  line: &wezterm_term::Line,
  output: &mut String,
  state: &mut AttrState,
  blank_attrs: &CellAttributes,
) {
  let cells: Vec<_> = line.visible_cells().collect();
  let mut last_col = 0usize;
  for cell in &cells {
    let is_blank = cell.str() == " " && cell.attrs() == blank_attrs;
    if !is_blank {
      last_col = cell.cell_index() + cell.width();
    }
  }
  let mut col = 0usize;
  for cell in cells {
    if cell.cell_index() >= last_col {
      break;
    }
    let target = cell.cell_index();
    if target > col {
      let gap_state = AttrState::from_attrs(blank_attrs);
      emit_attr_delta(output, state, gap_state);
      let gap = target.saturating_sub(col);
      for _ in 0..gap {
        output.push(' ');
      }
    }
    let next_state = AttrState::from_attrs(cell.attrs());
    emit_attr_delta(output, state, next_state);
    output.push_str(cell.str());
    col = target.saturating_add(cell.width());
  }
  if *state != AttrState::default() {
    output.push_str("\x1b[0m");
    *state = AttrState::default();
  }
}

fn line_has_content(line: &wezterm_term::Line, blank_attrs: &CellAttributes) -> bool {
  line
    .visible_cells()
    .any(|cell| cell.str() != " " || cell.attrs() != blank_attrs)
}

fn serialize_screen_to_ansi(
  screen: &wezterm_term::Screen,
  cursor_row: u16,
  cursor_col: u16,
) -> Vec<u8> {
  let total_rows = screen.scrollback_rows();
  if total_rows == 0 {
    return Vec::new();
  }
  let lines = screen.lines_in_phys_range(0..total_rows);
  let mut output = String::new();
  let mut state = AttrState::default();
  let blank_attrs = CellAttributes::blank();
  let mut last_content = None;
  for (index, line) in lines.iter().enumerate() {
    if line_has_content(line, &blank_attrs) {
      last_content = Some(index);
    }
  }
  let end = last_content.map(|index| index + 1).unwrap_or(0);
  if end == 0 {
    let row = cursor_row.saturating_add(1);
    let col = cursor_col.saturating_add(1);
    output.push_str(&format!("\x1b[{row};{col}H"));
    return output.into_bytes();
  }
  for (index, line) in lines.iter().take(end).enumerate() {
    if index > 0 {
      output.push_str("\r\n");
    }
    serialize_line_to_ansi(line, &mut output, &mut state, &blank_attrs);
  }
  if state != AttrState::default() {
    output.push_str("\x1b[0m");
  }
  let row = cursor_row.saturating_add(1);
  let col = cursor_col.saturating_add(1);
  output.push_str(&format!("\x1b[{row};{col}H"));
  output.into_bytes()
}
