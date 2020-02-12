use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{BarChart, Block, Borders, Paragraph, Text, Widget},
    Frame,
};
const PITCHES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub fn draw<B>(f: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    if let Some(analysis) = &app.audio_analysis {
        let progress_seconds = (app.song_progress_ms as f32) / 1000.0;

        let beat = analysis
            .beats
            .iter()
            .find(|beat| beat.start >= progress_seconds);

        let beat_offset = beat
            .map(|beat| beat.start - progress_seconds)
            .unwrap_or(0.0);
        let segment = analysis
            .segments
            .iter()
            .find(|segment| segment.start >= progress_seconds);
        let section = analysis
            .sections
            .iter()
            .find(|section| section.start >= progress_seconds);

        if let (Some(segment), Some(section)) = (segment, section) {
            let data: Vec<(&str, u64)> = segment
                .pitches
                .iter()
                .enumerate()
                .map(|(index, pitch)| {
                    let display_pitch = *PITCHES.get(index).unwrap_or(&PITCHES[0]);
                    let bar_value = ((pitch * 1000.0) as u64)
                        // Add a beat offset to make the bar animate between beats
                        .checked_add((beat_offset * 3000.0) as u64)
                        .unwrap_or(0);

                    (display_pitch, bar_value)
                })
                .collect();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
                .margin(2)
                .split(f.size());

            let analysis_block = Block::default()
                .title("Analysis")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.user_config.theme.inactive))
                .title_style(Style::default().fg(app.user_config.theme.inactive));

            Paragraph::new(
                [
                    Text::raw(format!(
                        "Tempo: {} (confidence {:.0}%)\n",
                        section.tempo,
                        section.tempo_confidence * 100.0
                    )),
                    Text::raw(format!(
                        "Key: {} (confidence {:.0}%)\n",
                        PITCHES.get(section.key as usize).unwrap_or(&PITCHES[0]),
                        section.key_confidence * 100.0
                    )),
                    Text::raw(format!(
                        "Time Signature: {}/4 (confidence {:.0}%)\n",
                        section.time_signature,
                        section.time_signature_confidence * 100.0
                    )),
                ]
                .iter(),
            )
            .block(analysis_block)
            .style(Style::default().fg(app.user_config.theme.text))
            .render(f, chunks[0]);

            let white = Style::default().fg(app.user_config.theme.text);
            let gray = Style::default().fg(app.user_config.theme.inactive);
            let width = f.size().width / PITCHES.len() as u16;
            BarChart::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(white)
                        .title(&format!(
                            "Pitches | Tick Rate {}",
                            app.user_config.behavior.tick_rate
                        ))
                        .title_style(gray)
                        .border_style(gray),
                )
                .data(&data)
                .bar_width(width)
                .bar_gap(1)
                .style(Style::default().fg(app.user_config.theme.analysis_bar))
                .value_style(
                    Style::default()
                        .fg(app.user_config.theme.analysis_bar_text)
                        .bg(app.user_config.theme.analysis_bar),
                )
                .render(f, chunks[1]);
        };
    }
}
