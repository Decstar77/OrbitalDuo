use std::any::Any;

use crate::{types::*, State};

use super::renderer::RenderState;

pub trait UIElement: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn compute_sizes(&mut self, parent_size: Option<Vec2>, rs: &mut RenderState) -> Vec2;
    fn update_and_render(&mut self, bounds: Bounds, state: &mut State);
}

pub enum UISize {
    Pixels((f32, f32)),
    PercentOfParent((f32, f32)),
}

pub struct UIButton {
    pub text: String,
    pub padding: Vec2,
    pub on_click: Box<dyn Fn(&mut State)>,
    pub on_hover: Box<dyn Fn(&mut State)>,
    is_hoverd: bool,
    computed_size: Vec2,
}

impl UIElement for UIButton {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn compute_sizes(&mut self, parent_size: Option<Vec2>, rs: &mut RenderState) -> Vec2 {
        let w = rs.get_text_width(&self.text);
        let h = rs.get_text_height(&self.text);
        self.computed_size = Vec2::new(w, h) + self.padding * 2.0;
        self.computed_size
    }

    fn update_and_render(&mut self, bounds: Bounds, state: &mut State) {
        let color = Vec4::new(0.192, 0.215, 0.235, 1.0);
        let hover_color = color * 2.0;
        let click_color = color * 3.0;

        let mut use_color = color;

        if bounds.contains(state.fs.mouse_pos) {
            if self.is_hoverd == false {
                (self.on_hover)(state);
            }
            
            self.is_hoverd = true;

            use_color = hover_color;
            if state.fs.is_mouse_pressed(0) {
                use_color = click_color;
            }

            if state.fs.is_mouse_just_released(0) {
                (self.on_click)(state);
            }
        } else {
            self.is_hoverd = false;
        }

        state.rs.draw_rect_min_max(bounds.min, bounds.max)
            .with_color(use_color);

        let text_pos =
            bounds.min + Vec2::new(self.padding.x, self.computed_size.y - self.padding.y);

        state.rs.draw_text(&self.text, text_pos);
    }
}

impl UIButton {
    pub fn new(text: &str) -> UIButton {
        UIButton {
            text: text.to_string(),
            padding: Vec2::new(7.0, 7.0),
            on_click: Box::new(|state| {
                state.ad.play_sound("sfxD05");
            }),
            on_hover: Box::new(|state| {
                state.ad.play_sound("sfxD03")
            }),
            is_hoverd: false,
            computed_size: Vec2::new(0.0, 0.0),
        }
    }

    pub fn new_callback(text: &str, on_click: Box<dyn Fn(&mut State)>) -> UIButton {
        UIButton {
            text: text.to_string(),
            padding: Vec2::new(7.0, 7.0),
            on_click,
            on_hover: Box::new(|state| {
                state.ad.play_sound("sfxD03")
            }),
            is_hoverd: false,
            computed_size: Vec2::new(0.0, 0.0),
        }
    }
}

pub enum UIBlockContainerXConstraint {
    None,
    LEFT,
    CENTER,
    RIGHT,
}

pub enum UIBlockContainerYConstraint {
    None,
    TOP,
    CENTER,
    BOTTOM,
}

pub struct UIBlockContainerContraints {
    pub x_constraint: UIBlockContainerXConstraint,
    pub y_constraint: UIBlockContainerYConstraint,
}

pub struct UIBlockContainerChild {
    element: Box<dyn UIElement>,
    constraints: UIBlockContainerContraints,
    size: Vec2,
}

pub struct UIBlockContainer {
    size: UISize,
    children: Vec<UIBlockContainerChild>,

    computed_size: Vec2,
}

impl UIElement for UIBlockContainer {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn compute_sizes(&mut self, parent_size: Option<Vec2>, rs: &mut RenderState) -> Vec2 {
        match self.size {
            UISize::Pixels(pixels) => {
                self.computed_size = Vec2::new(pixels.0, pixels.1);

                for child in &mut self.children {
                    child.size = child.element.compute_sizes(Some(self.computed_size), rs);
                }

                self.computed_size
            }
            UISize::PercentOfParent(pertcent) => {
                let parent_size = parent_size.expect("PercentOfParent needs parent size");

                self.computed_size =
                    Vec2::new(parent_size.x * pertcent.0, parent_size.y * pertcent.1);

                for child in &mut self.children {
                    child.size = child.element.compute_sizes(Some(self.computed_size), rs);
                }

                self.computed_size
            }
        }
    }

    fn update_and_render(&mut self, bounds: Bounds, state: &mut State) {
        for child in &mut self.children {
            let mut pos = bounds.min;

            match child.constraints.x_constraint {
                UIBlockContainerXConstraint::None => {}
                UIBlockContainerXConstraint::LEFT => {}
                UIBlockContainerXConstraint::CENTER => {
                    pos.x += self.computed_size.x / 2.0 - child.size.x / 2.0;
                }
                UIBlockContainerXConstraint::RIGHT => {
                    pos.x += self.computed_size.x - child.size.x;
                }
            }

            match child.constraints.y_constraint {
                UIBlockContainerYConstraint::None => {}
                UIBlockContainerYConstraint::TOP => {}
                UIBlockContainerYConstraint::CENTER => {
                    pos.y += self.computed_size.y / 2.0 - child.size.y / 2.0;
                }
                UIBlockContainerYConstraint::BOTTOM => {
                    pos.y += self.computed_size.y - child.size.y;
                }
            }

            let child_bounds = Bounds {
                min: pos,
                max: pos + child.size,
            };

            //rs.draw_rect_min_max(child_bounds.min, child_bounds.max);

            child.element.update_and_render(child_bounds, state);
        }
    }
}

impl UIBlockContainer {
    pub fn new_from_pixels(w: f32, h: f32) -> UIBlockContainer {
        UIBlockContainer {
            size: UISize::Pixels((w, h)),
            children: Vec::new(),
            computed_size: Vec2::new(0.0, 0.0),
        }
    }

    pub fn new_from_percent(w: f32, h: f32) -> UIBlockContainer {
        UIBlockContainer {
            size: UISize::PercentOfParent((w, h)),
            children: Vec::new(),
            computed_size: Vec2::new(0.0, 0.0),
        }
    }

    pub fn add_child(
        &mut self,
        child: Box<dyn UIElement>,
        constraints: UIBlockContainerContraints,
    ) {
        self.children.push(UIBlockContainerChild {
            element: child,
            constraints: constraints,
            size: Vec2::new(0.0, 0.0),
        });
    }
}

pub enum UIStackPaneOrientation {
    HORIZONTAL,
    VERTICAL,
}

pub enum UIStackPaneChildAlignment {
    START,
    END,
}

pub struct UIStackPaneChild {
    element: Box<dyn UIElement>,
    size: Vec2,
}

pub struct UIStackPaneContainer {
    orientation: UIStackPaneOrientation,
    child_alignment: UIStackPaneChildAlignment,
    children: Vec<UIStackPaneChild>,
    computed_size: Vec2,
    computed_pos: Vec2,
}

impl UIElement for UIStackPaneContainer {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn compute_sizes(&mut self, parent_size: Option<Vec2>, rs: &mut RenderState) -> Vec2 {
        let mut total_size = Vec2::new(0.0, 0.0);

        for child in &mut self.children {
            let child_size = child.element.compute_sizes(parent_size, rs);
            child.size = child_size;

            match self.orientation {
                UIStackPaneOrientation::HORIZONTAL => {
                    total_size.x += child_size.x;
                    total_size.y = total_size.y.max(child_size.y);
                }
                UIStackPaneOrientation::VERTICAL => {
                    total_size.x = total_size.x.max(child_size.x);
                    total_size.y += child_size.y;
                }
            }
        }

        self.computed_size = total_size;
        self.computed_size
    }

    fn update_and_render(&mut self, bounds: Bounds, state: &mut State) {
        let mut pos = bounds.min;
        for child in &mut self.children {
            let mut child_bounds = Bounds::new(pos, pos + child.size);

            match self.orientation {
                UIStackPaneOrientation::HORIZONTAL => {
                    child_bounds.max.y = bounds.max.y;
                }
                UIStackPaneOrientation::VERTICAL => {
                    child_bounds.max.x = bounds.max.x;
                }
            }

            child.element.update_and_render(child_bounds, state);

            match self.orientation {
                UIStackPaneOrientation::HORIZONTAL => {
                    pos.x += child.size.x;
                }
                UIStackPaneOrientation::VERTICAL => {
                    pos.y += child.size.y;
                }
            }
        }
    }
}

impl UIStackPaneContainer {
    pub fn new() -> UIStackPaneContainer {
        UIStackPaneContainer {
            orientation: UIStackPaneOrientation::HORIZONTAL,
            child_alignment: UIStackPaneChildAlignment::START,
            children: Vec::new(),
            computed_size: Vec2::new(0.0, 0.0),
            computed_pos: Vec2::new(0.0, 0.0),
        }
    }

    pub fn new_vertical() -> UIStackPaneContainer {
        UIStackPaneContainer {
            orientation: UIStackPaneOrientation::VERTICAL,
            child_alignment: UIStackPaneChildAlignment::START,
            children: Vec::new(),
            computed_size: Vec2::new(0.0, 0.0),
            computed_pos: Vec2::new(0.0, 0.0),
        }
    }

    pub fn add_child(&mut self, child: Box<dyn UIElement>) {
        self.children.push(UIStackPaneChild {
            element: child,
            size: Vec2::new(0.0, 0.0),
        });
    }
}
pub struct UIMaster {
    size: Vec2,
    screen: UIBlockContainer,
}

impl UIMaster {
    pub fn new() -> UIMaster {
        UIMaster {
            size: Vec2::new(0.0, 0.0),
            screen: UIBlockContainer::new_from_pixels(0.0, 0.0),
        }
    }

    pub fn add_child(
        &mut self,
        child: Box<dyn UIElement>,
        constraints: UIBlockContainerContraints,
    ) {
        self.screen.add_child(child, constraints);
    }

    pub fn update_and_render(&mut self, state: &mut State) {
        let fs = &mut state.fs;
        let rs = &mut state.rs;

        self.size = Vec2::new(rs.surface_width as f32, rs.surface_height as f32);
        self.screen.size = UISize::Pixels((self.size.x, self.size.y));
        self.screen.compute_sizes(None, rs);

        let bounds = Bounds {
            min: Vec2::new(0.0, 0.0),
            max: self.size,
        };

        self.screen.update_and_render(bounds, state);
    }
}
