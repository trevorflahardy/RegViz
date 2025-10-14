use std::collections::HashMap;

use iced::widget::canvas::{self, Path, Stroke, Style, Text as CanvasText};
use iced::{Color, Point, Rectangle, Size, Theme, Vector, mouse};

use crate::app::Message;
use crate::core::BuildArtifacts;
use crate::core::nfa::{EdgeLabel, Nfa};
use crate::viz::layout::layout_states;

const STATE_RADIUS: f32 = 26.0;

pub struct AutomatonCanvas<'a> {
    artifacts: &'a BuildArtifacts,
}

impl<'a> AutomatonCanvas<'a> {
    pub fn new(artifacts: &'a BuildArtifacts) -> Self {
        Self { artifacts }
    }
}

impl<'a> canvas::Program<Message> for AutomatonCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(_renderer, bounds.size());
        let positions = state_positions(&self.artifacts.nfa, bounds.size());
        draw_edges(&mut frame, &self.artifacts.nfa, &positions);
        draw_states(&mut frame, &self.artifacts.nfa, &positions);
        vec![frame.into_geometry()]
    }
}

fn state_positions(nfa: &Nfa, size: Size) -> HashMap<u32, Point> {
    layout_states(nfa, size.width, size.height)
        .into_iter()
        .collect()
}

fn draw_edges(frame: &mut canvas::Frame, nfa: &Nfa, positions: &HashMap<u32, Point>) {
    for edge in &nfa.edges {
        let Some(&from) = positions.get(&edge.from) else {
            continue;
        };
        let Some(&to) = positions.get(&edge.to) else {
            continue;
        };
        if edge.from == edge.to {
            draw_self_loop(frame, from, label_for(edge.label));
            continue;
        }
        let path = Path::line(from, to);
        frame.stroke(
            &path,
            Stroke {
                width: 2.0,
                style: Style::Solid(Color::WHITE),
                ..Stroke::default()
            },
        );
        draw_arrow_head(frame, from, to);
        let mid = Point::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
        let offset = Point::new(mid.x, mid.y - 8.0);
        frame.fill_text(CanvasText {
            content: label_for(edge.label),
            position: offset,
            color: Color::WHITE,
            size: iced::Pixels(16.0),
            ..CanvasText::default()
        });
    }
}

fn draw_states(frame: &mut canvas::Frame, nfa: &Nfa, positions: &HashMap<u32, Point>) {
    for state in &nfa.states {
        if let Some(&pos) = positions.get(state) {
            let circle = Path::circle(pos, STATE_RADIUS);
            frame.stroke(
                &circle,
                Stroke {
                    width: 2.0,
                    style: Style::Solid(Color::WHITE),
                    ..Stroke::default()
                },
            );
            if nfa.accepts.contains(state) {
                let inner = Path::circle(pos, STATE_RADIUS - 6.0);
                frame.stroke(
                    &inner,
                    Stroke {
                        width: 2.0,
                        style: Style::Solid(Color::WHITE),
                        ..Stroke::default()
                    },
                );
            }
            if *state == nfa.start {
                let start_from = Point::new(pos.x - STATE_RADIUS - 30.0, pos.y);
                let path = Path::line(start_from, Point::new(pos.x - STATE_RADIUS, pos.y));
                frame.stroke(
                    &path,
                    Stroke {
                        width: 2.0,
                        style: Style::Solid(Color::WHITE),
                        ..Stroke::default()
                    },
                );
                draw_arrow_head(frame, start_from, Point::new(pos.x - STATE_RADIUS, pos.y));
            }
            frame.fill_text(CanvasText {
                content: format!("q{}", state),
                position: Point::new(pos.x - 10.0, pos.y + 5.0),
                color: Color::WHITE,
                size: iced::Pixels(18.0),
                ..CanvasText::default()
            });
        }
    }
}

fn draw_arrow_head(frame: &mut canvas::Frame, from: Point, to: Point) {
    let direction = Vector::new(to.x - from.x, to.y - from.y);
    let length = (direction.x.powi(2) + direction.y.powi(2)).sqrt();
    if length == 0.0 {
        return;
    }
    let unit = Vector::new(direction.x / length, direction.y / length);
    let normal = Vector::new(-unit.y, unit.x);
    let tip = to;
    let back = Point::new(tip.x - unit.x * 12.0, tip.y - unit.y * 12.0);
    let left = Point::new(back.x + normal.x * 6.0, back.y + normal.y * 6.0);
    let right = Point::new(back.x - normal.x * 6.0, back.y - normal.y * 6.0);
    let path = Path::new(|builder| {
        builder.move_to(tip);
        builder.line_to(left);
        builder.line_to(right);
        builder.close();
    });
    frame.fill(&path, Color::from_rgb(0.7, 0.7, 0.7));
}

fn draw_self_loop(frame: &mut canvas::Frame, center: Point, label: String) {
    let offset = Vector::new(0.0, -STATE_RADIUS - 30.0);
    let loop_center = Point::new(center.x + offset.x, center.y + offset.y);
    let path = Path::circle(loop_center, 18.0);
    frame.stroke(
        &path,
        Stroke {
            width: 2.0,
            style: Style::Solid(Color::from_rgb(0.7, 0.7, 0.7)),
            ..Stroke::default()
        },
    );
    frame.fill_text(CanvasText {
        content: label,
        position: Point::new(loop_center.x - 6.0, loop_center.y - 20.0),
        color: Color::WHITE,
        size: iced::Pixels(16.0),
        ..CanvasText::default()
    });
}

fn label_for(label: EdgeLabel) -> String {
    match label {
        EdgeLabel::Eps => "ε".into(),
        EdgeLabel::Sym(c) => c.to_string(),
    }
}
