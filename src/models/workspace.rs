#![allow(dead_code)]
use indexmap::IndexMap;

use crate::{
    config::*,
    layout::{self, column_master, floating, Layout, LayoutTag},
    models::windowwrapper::WindowWrapper,
    xlibwrapper::xlibmodels::Window,
};

#[derive(Debug)]
pub struct Workspace {
    pub tag: u32,
    pub clients: IndexMap<Window, WindowWrapper>,
    pub layout: Box<dyn Layout>,
    available_layouts: Vec<LayoutTag>,
    current_tag: LayoutTag,
}

impl Workspace {
    pub fn new(tag: u32) -> Self {
        Self {
            tag,
            clients: Default::default(),
            layout: layout::layout_from_tag(CONFIG.default_layout),
            available_layouts: vec![LayoutTag::Floating, LayoutTag::ColumnMaster],
            current_tag: CONFIG.default_layout,
        }
    }

    pub fn get_current_layout(&self) -> LayoutTag {
        self.current_tag
    }

    pub fn circulate_layout(&mut self) {
        let index = self
            .available_layouts
            .iter()
            .position(|lt| self.current_tag == *lt)
            .unwrap()
            + 1;

        let index = index % self.available_layouts.len();
        self.current_tag = self.available_layouts[index];
        self.layout = match self.current_tag {
            LayoutTag::Floating => Box::new(floating::Floating::default()),
            LayoutTag::ColumnMaster => Box::new(column_master::ColumnMaster::default()),
        }
    }

    pub fn contains_window(&self, w: Window) -> bool {
        self.clients.contains_key(&w)
    }

    pub fn add_window(&mut self, w: Window, ww: WindowWrapper) {
        self.clients.insert(w, ww);
        self.clients.sort_by(|_ka, va, _kb, vb| va.toc.cmp(&vb.toc));
        debug!("window order: {:?}", self.clients.keys().collect::<Vec<&Window>>());
    }

    pub fn remove_window(&mut self, w: Window) -> Option<WindowWrapper> {
        let ret = self.clients.shift_remove(&w);
        self.clients.sort_by(|_ka, va, _kb, vb| va.toc.cmp(&vb.toc));
        ret
    }

    pub fn get_newest(&self) -> Option<(&Window, &WindowWrapper)> {
        self.clients.iter().last()
    }

    pub fn get_previous(&self, ww: &WindowWrapper) -> Option<&WindowWrapper> {
        if self.clients.len() <= 1 {
            return None;
        }
        match self.clients.get_full(&ww.window()) {
            Some((mut index, ..)) => {
                if index == 0 {
                    index = self.clients.len() - 2;
                } else {
                    index -= 1;
                }
                let ret = self.clients.get_index(index)?.1;

                if ret.window() == ww.window() {
                    return None;
                } else {
                    return Some(ret);
                }
            }
            _ => None,
        }
    }

    pub fn get_next(&self, ww: &WindowWrapper) -> Option<&WindowWrapper> {
        if self.clients.len() <= 1 {
            return None;
        }
        match self.clients.get_full(&ww.window()) {
            Some((mut index, ..)) => {
                if index == self.clients.len() - 2 || index == self.clients.len() - 1 {
                    index = 0;
                } else {
                    index += 1;
                }
                let ret = self.clients.get_index(index)?.1;

                if ret.window() == ww.window() {
                    return None;
                } else {
                    return Some(ret);
                }
            }
            _ => None,
        }
    }
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}

impl Eq for Workspace {}
