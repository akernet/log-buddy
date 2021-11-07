use gtk::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use gtk::{
    *
};

pub struct SidebarFileList {
    pub tree_view: TreeView,
    store: TreeStore
}

impl SidebarFileList {
    pub fn new() -> Self {
        let tree = TreeView::builder()
            .build();

        let store = TreeStore::new(&[String::static_type(), bool::static_type(), bool::static_type()]);

        let col = TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        col.set_title("File Name");
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 0);
        col.set_expand(true);
        tree.append_column(&col);

        let col = TreeViewColumn::new();
        let cell = gtk::CellRendererToggle::new();
        col.set_title("Show");
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "active", 2);
        col.add_attribute(&cell, "visible", 1);
        tree.append_column(&col);

        tree.set_model(Some(&store));

        Self {
            tree_view: tree,
            store: store,
        }
    }

    pub fn update(&self, path_list: &Vec<PathBuf>) {
        let store = &self.store;
        store.clear();

        let root = store.insert_with_values(None, Some(0), &[(0, &"/"), (1, &false), (2, &false)]);

        struct FileTree {
            iter: TreeIter,
            children: HashMap<String, Self>
        }

        let mut root = FileTree{
            iter: root,
            children: HashMap::new()
        };

        for path in path_list {
            let mut prev = &mut root;

            let count = path.components().count();
            for (index, component) in path.components().enumerate() {
                match component {
                    std::path::Component::RootDir => {},
                    std::path::Component::Normal(name) => {
                        let name = name.to_str().unwrap();

                        match prev.children.get(name) {
                            None => {
                                let last = index == count - 1;
                                prev.children.insert(name.to_owned(), FileTree{
                                    iter: store.insert_with_values(Some(&prev.iter), Some(0), &[(0, &name), (1, &last), (2, &last)]),
                                    children: HashMap::new()
                                });
                            },
                            _ => {}
                        }

                        prev = prev.children.get_mut(name).unwrap();
                    },
                    _ => panic!("Unknown component type")
                }

            }

        }

        self.tree_view.expand_all();

    }
}