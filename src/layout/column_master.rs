#![allow(dead_code, unused_variables)]
use super::*;
use crate::{
    config::*,
    models::{
        dockarea::DockArea, rect::Rect, screen::Screen, windowwrapper::WindowWrapper, Direction,
    },
    xlibwrapper::{
        util::{Position, Size},
        xlibmodels::*,
    },
};
use std::collections::LinkedList;

#[derive(Debug)]
pub struct ColumnMaster {
    layout_type: LayoutTag,
    column: LinkedList<Window>,
}

impl ColumnMaster {
    fn get_size(ww: &WindowWrapper, size: (Size, Size)) -> Size {
        if ww.is_decorated() {
            size.0
        } else {
            size.1
        }
    }

    fn column_maximize(
        &self,
        w: Window,
        screen: &Screen,
        dock_area: &DockArea,
    ) -> (Size, Position) {
        match dock_area.as_rect(screen) {
            Some(dock) => (
                Size {
                    width: screen.width - 2 * CONFIG.border_width,
                    height: screen.height - dock.get_size().height - 2 * CONFIG.border_width,
                },
                Position {
                    x: screen.x,
                    y: screen.y + dock.get_size().height,
                },
            ),
            None => (
                Size {
                    width: screen.width - 2 * CONFIG.border_width,
                    height: screen.height + 2 * CONFIG.border_width,
                },
                Position {
                    x: screen.x,
                    y: screen.y,
                },
            ),
        }
    }

    fn update_column(&mut self, w: Window, windows: &mut Vec<&WindowWrapper>) {
        let windows_keys = windows
            .iter()
            .map(|win| win.window())
            .collect::<Vec<Window>>();
        let new_column = self
            .column
            .iter()
            .map(|win| *win)
            .filter(|win| windows_keys.contains(win))
            .collect::<LinkedList<Window>>();
        self.column = new_column;
        self.column.push_front(w);
    }

    fn column_height(&self, screen: &Screen, dock: &DockArea) -> i32 {
        let dock_height = match dock.as_rect(screen) {
            Some(dock_rect) => dock_rect.get_size().height,
            None => 0,
        };

        ((screen.height - dock_height)
            / if self.column.len() > 0 {
                self.column.len() as i32
            } else {
                1
            })
            - 2 * CONFIG.border_width
    }
}

impl Default for ColumnMaster {
    fn default() -> Self {
        Self {
            layout_type: LayoutTag::ColumnMaster,
            column: LinkedList::new(),
        }
    }
}

impl std::fmt::Display for ColumnMaster {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.layout_type)
    }
}

impl Layout for ColumnMaster {
    fn place_window(
        &mut self,
        dock_area: &DockArea,
        screen: &Screen,
        w: Window,
        mut windows: Vec<&WindowWrapper>,
    ) -> Vec<(Window, Rect)> {
        self.update_column(w, &mut windows);
        let dock_height = match dock_area.as_rect(screen) {
            Some(dock_rect) => dock_rect.get_size().height,
            None => 0,
        };

        let mut ret_vec = Vec::<(Window, Rect)>::new();

        let column_width = (screen.width / 2) - 2 * CONFIG.border_width;
        let column_x = (screen.width / 2) + CONFIG.border_width;
        

        if self.column.len() == 1 {
            let (size, pos) = self.column_maximize(w, &screen, &dock_area);
            ret_vec.push((w, Rect::new(pos, size)));
        } else {
            let size = Size {
                width: column_width,
                height: screen.height - dock_height,
            };
            let pos = Position {
                x: screen.x,
                y: screen.y + dock_height,
            };
            let first = self.column.front().unwrap();
            ret_vec.push((*first, Rect::new(pos, size)));
            let _ = self
                .column
                .iter()
                .skip(1)
                .map(|win| {
                    (
                        *win,
                        Rect::new(
                            Position {
                                x: column_x,
                                y: screen.y + dock_height,
                            },
                            Size {
                                width: column_width,
                                height: self.column_height(&screen, &dock_area),
                            },
                        ),
                    )
                })
                .for_each(|tup| ret_vec.push(tup));
        debug!("column_master column: {:?}", self.column);
        }
        ret_vec
    }

    fn place_window_relative(
        &self,
        dock_area: &DockArea,
        screen: &Screen,
        w: Window,
        ww: &WindowWrapper,
    ) -> (Size, Position) {
        unimplemented!();
    }

    fn move_window(
        &self,
        screen: &Screen,
        dock_area: &DockArea,
        w: Window,
        respect_dock: bool,
        x: i32,
        y: i32,
    ) -> (Position, Position) {
        let mut y = y;
        match dock_area.as_rect(&screen) {
            Some(dock) if respect_dock => {
                if y < dock.get_size().height as i32 {
                    y = dock.get_size().height as i32 - CONFIG.border_width;
                }
            }
            Some(_) | None => {}
        }

        (
            Position { x, y },
            Position {
                x: x + CONFIG.border_width,
                y: y + CONFIG.decoration_height + CONFIG.border_width,
            },
        )
    }

    fn reorder(
        &mut self,
        focus: Window,
        screen: &Screen,
        dock_area: &DockArea,
        windows: Vec<WindowWrapper>,
    ) -> Vec<Rect> {
        vec![]
    }

    fn resize_window(
        &self,
        ww: &WindowWrapper,
        w: Window,
        width: i32,
        height: i32,
    ) -> (Size, Size) {
        println!("width: {}", width);

        println!("is window decorated: {}", ww.is_decorated());
        if let Some(dec_rect) = ww.get_dec_rect() {
            let dec_size = Size { width, height };
            let window_size = Size {
                width,
                height: height - CONFIG.decoration_height,
            };
            (dec_size, window_size)
        } else {
            let dec_size = Size {
                width: 0,
                height: 0,
            };
            let window_size = Size { width, height };
            (dec_size, window_size)
        }
    }

    fn maximize(
        &self,
        screen: &Screen,
        dock_area: &DockArea,
        ww: &WindowWrapper,
        w: Window,
    ) -> (Position, Size) {
        let pos = self.move_window(screen, dock_area, w, true, screen.x, screen.y);
        match dock_area.as_rect(&screen) {
            Some(dock) => {
                let size = ColumnMaster::get_size(
                    &ww,
                    self.resize_window(
                        &ww,
                        w,
                        screen.width,
                        screen.height - dock.get_size().height,
                    ),
                );
                (pos.0, size)
            }
            None => {
                let size = ColumnMaster::get_size(
                    &ww,
                    self.resize_window(&ww, w, screen.width, screen.height),
                );
                (pos.0, size)
            }
        }
    }

    fn monocle(
        &self,
        screen: &Screen,
        dock_area: &DockArea,
        ww: &WindowWrapper,
        w: Window,
    ) -> (Position, Size) {
        // TODO: implement for decorated windows
        let pos = self.move_window(screen, dock_area, w, false, screen.x, screen.y);
        let size =
            ColumnMaster::get_size(&ww, self.resize_window(&ww, w, screen.width, screen.height));
        (pos.0, size)
    }

    fn shift_window(
        &self,
        screen: &Screen,
        ww: &WindowWrapper,
        dock_area: &DockArea,
        w: Window,
        direction: Direction,
    ) -> Vec<WindowWrapper> {
        let origin = Position {
            x: screen.x,
            y: screen.y,
        };
        match direction {
            Direction::North => {
                let pos = self
                    .move_window(&screen, &dock_area, w, true, screen.x, screen.y)
                    .0;
                let size = if ww.is_decorated() {
                    Size {
                        width: screen.width - 2 * CONFIG.border_width,
                        height: (screen.height) / 2 - 2 * CONFIG.border_width,
                    }
                } else {
                    Size {
                        width: screen.width - 2 * CONFIG.border_width,
                        height: (screen.height) / 2 - 2 * CONFIG.border_width,
                    }
                };
                let mut size = ColumnMaster::get_size(
                    &ww,
                    self.resize_window(&ww, w, size.width, size.height),
                );
                if let Some(dock) = dock_area.as_rect(&screen) {
                    size.height -= dock.get_size().height / 2
                }
                vec![WindowWrapper {
                    window_rect: Rect::new(pos, size),
                    ..ww.clone()
                }]
            }
            Direction::East => {
                let pos = self
                    .move_window(
                        &screen,
                        &dock_area,
                        w,
                        true,
                        screen.x + screen.width / 2,
                        screen.y,
                    )
                    .0;
                let size = if ww.is_decorated() {
                    Size {
                        width: (screen.width) / 2 - 2 * CONFIG.border_width,
                        height: screen.height - CONFIG.border_width,
                    }
                } else {
                    Size {
                        width: (screen.width) / 2 - 2 * CONFIG.border_width,
                        height: screen.height - CONFIG.border_width,
                    }
                };
                let mut size = ColumnMaster::get_size(
                    &ww,
                    self.resize_window(&ww, w, size.width, size.height),
                );
                if let Some(dock) = dock_area.as_rect(&screen) {
                    size.height -= dock.get_size().height;
                }
                vec![WindowWrapper {
                    window_rect: Rect::new(pos, size),
                    ..ww.clone()
                }]
            }
            Direction::West => {
                let pos = self
                    .move_window(&screen, &dock_area, w, true, screen.x, screen.y)
                    .0;
                let size = if ww.is_decorated() {
                    Size {
                        width: (screen.width) / 2 - 2 * CONFIG.border_width,
                        height: (screen.height) - CONFIG.border_width,
                    }
                } else {
                    Size {
                        width: (screen.width) / 2 - 2 * CONFIG.border_width,
                        height: (screen.height) - CONFIG.border_width,
                    }
                };
                let mut size = ColumnMaster::get_size(
                    &ww,
                    self.resize_window(&ww, w, size.width, size.height),
                );
                if let Some(dock) = dock_area.as_rect(&screen) {
                    size.height -= dock.get_size().height;
                }
                vec![WindowWrapper {
                    window_rect: Rect::new(pos, size),
                    ..ww.clone()
                }]
            }
            Direction::South => {
                let mut pos = self
                    .move_window(
                        &screen,
                        &dock_area,
                        w,
                        true,
                        screen.x,
                        screen.height / 2 - CONFIG.border_width,
                    )
                    .0;
                let size = if ww.is_decorated() {
                    Size {
                        width: screen.width - 2 * CONFIG.border_width,
                        height: (screen.height) / 2 - 2 * CONFIG.border_width,
                    }
                } else {
                    Size {
                        width: screen.width - 2 * CONFIG.border_width,
                        height: (screen.height) / 2 - CONFIG.border_width,
                    }
                };
                let mut size = ColumnMaster::get_size(
                    &ww,
                    self.resize_window(&ww, w, size.width, size.height),
                );
                if let Some(dock) = dock_area.as_rect(&screen) {
                    let offset = dock.get_size().height / 2;
                    size.height -= offset;
                    pos.y += offset as i32;
                }
                vec![WindowWrapper {
                    window_rect: Rect::new(pos, size),
                    ..ww.clone()
                }]
            }
        }
    }
}
