use iced_winit::winit;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, ModifiersState, MouseButton, WindowEvent},
};

#[derive(Clone, Copy, Debug)]
pub struct DrawArea {
    pub position: PhysicalPosition<u32>,
    pub size: PhysicalSize<u32>,
}

pub struct ScreenMultiplexer {
    modifiers: ModifiersState,
    window_size: PhysicalSize<u32>,
    left_focus: bool,
    mouse_clicked: bool,
    partition: u32,
    left: Side,
    right: Side,
}

impl ScreenMultiplexer {
    /// `partition`: Size of the left side in pixels
    pub fn new(partition: u32, initial_size: PhysicalSize<u32>) -> Self {
        Self {
            partition,
            modifiers: ModifiersState::default(),
            window_size: initial_size,
            left_focus: true,
            mouse_clicked: false,
            left: Default::default(),
            right: Default::default(),
        }
    }

    /// Viewports for left and right partitions
    pub fn areas(&self) -> (DrawArea, DrawArea) {
        (
            DrawArea {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(self.partition, self.window_size.height),
            },
            DrawArea {
                position: PhysicalPosition::new(self.partition, 0),
                size: PhysicalSize::new(
                    self.window_size.width - self.partition,
                    self.window_size.height,
                ),
            },
        )
    }

    /// Cursors for left and right partitions
    pub fn cursors(&self) -> (PhysicalPosition<f64>, PhysicalPosition<f64>) {
        (self.left.cursor_position, self.right.cursor_position)
    }

    /// Modifiers for the curent cursor. Not split because it should be the same for both. This
    /// structure just tracks this for clarity and maybe futureproofing.
    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }

    /// Handle an event, and dole out events to the left and right partitions.
    pub fn event(
        &mut self,
        event: WindowEvent<'static>,
    ) -> (Option<WindowEvent<'static>>, Option<WindowEvent<'static>>) {
        match &event {
            WindowEvent::CursorMoved { position, .. } => {
                let &PhysicalPosition { x, y } = position;
                if x > 0.0 || y > 0.0 {
                    let (left_area, right_area) = self.areas();
                    if !self.mouse_clicked {
                        self.left_focus =
                            left_area.inside(PhysicalPosition::new(x as u32, y as u32));
                    }
                    if self.left_focus {
                        self.left.cursor_position = *position;
                    } else {
                        self.right.cursor_position = *position;
                        self.right.cursor_position.x -= right_area.position.x as f64;
                        self.right.cursor_position.y -= right_area.position.y as f64;
                    }
                }
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = *new_modifiers;
            }
            WindowEvent::Resized(new_size) => {
                self.window_size = *new_size;
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => match state {
                ElementState::Pressed => self.mouse_clicked = true,
                ElementState::Released => self.mouse_clicked = false,
            },
            _ => {}
        }

        if self.left_focus {
            (Some(event), None)
        } else {
            (None, Some(event))
        }
    }
}

impl DrawArea {
    fn inside(&self, position: PhysicalPosition<u32>) -> bool {
        position.x >= self.position.x
            && position.y >= self.position.y
            && position.x <= self.position.x + self.size.width
            && position.y <= self.position.y + self.size.height
    }
}

struct Side {
    cursor_position: PhysicalPosition<f64>,
}

impl Default for Side {
    fn default() -> Self {
        Self {
            cursor_position: PhysicalPosition::new(-1.0, -1.0),
        }
    }
}
