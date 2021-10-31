use gtk::prelude::*;
use gtk::*;
use glib::clone;
use std::rc::Rc;
use std::cell::Cell;
use tempfile::tempdir;

use compress_tools::*;
use std::fs::File;
use std::path::Path;

use std::io::{Write};

use walkdir::WalkDir;

fn append_column(tree: &TreeView, id: i32) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.set_sizing(TreeViewColumnSizing::Fixed);

    column.set_resizable(true);

    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "text", id);
    tree.append_column(&column);
}

fn build_ui(app: &Application) {

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Log Buddy")
        .default_height(600)
        .default_width(600)
        .build();


    let drop_target = DropTarget::builder().build();

    drop_target.connect_accept(|s, d| {
        println!("{:?} {:?}", s, d);
        println!("{:?}", d.formats().unwrap().types());
        println!("{:?}", d.actions());
        true
    });
    window.add_controller(&drop_target);

    println!("{:?}", drop_target.actions());


    let model = ListStore::new(&[u32::static_type(), String::static_type()]);

    let num_elements = Rc::new(Cell::new(0));

    let entries = &["Michel", "Sara", "Liam", "Zelda", "Neo", "Octopus master"];
    for (i, entry) in entries.repeat(1000).iter().enumerate() {
        model.insert_with_values(None, &[(0, &(i as u32 + 1)), (1, &entry)]);
        num_elements.set(num_elements.get()+1);
    }

    let tree_view = TreeView::builder()
        .model(&model)
        .reorderable(true)
        .fixed_height_mode(true)
        .build();

    tree_view.set_headers_visible(true);
    append_column(&tree_view, 0);
    append_column(&tree_view, 1);

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
        .min_content_width(360)
        .child(&tree_view)
        .kinetic_scrolling(true)
        .vexpand(true)
        .build();

    let button1 = Button::builder()
        .label("test")
        .build();
    let button2 = Button::builder()
        .label("reverse")
        .build();

    button1.connect_clicked(clone!(@weak model, @weak num_elements => move |_| {
        for (i, entry) in entries.clone().repeat(1_000_000).iter().enumerate() {
            model.insert_with_values(None, &[(0, &(i as u32 + 1)), (1, &entry)]);
            num_elements.set(num_elements.get()+1);
        }
    }));
    button2.connect_clicked(clone!(@weak model => move |_| {
        println!("Starting prep new order {}", num_elements.get());
        let mut new_order = (0..num_elements.get()).collect::<Vec<u32>>();
        new_order.reverse();
        println!("Done prep new order {}", num_elements.get());
        model.reorder(new_order.as_slice());
    }));

    let boks = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    boks.append(&scrolled_window);
    boks.append(&button1);
    boks.append(&button2);

    window.set_child(Some(&boks));

    window.present();
}

fn uncompress_recursive(path: &Path, dest: &Path) {
    let mut source = File::open(path).unwrap();
    uncompress_archive(&mut source, dest, Ownership::Ignore).unwrap();
    drop(source);

    let files = WalkDir::new(dest)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.metadata().unwrap().is_file());


    for entry in files {
        println!("{}", entry.path().display());
    }
}

fn main() {
    // Create a directory inside of `std::env::temp_dir()`.
    let tmp = tempdir().unwrap();

    let file_path = tmp.path().join("my-temporary-note.txt");
    let mut file = File::create(file_path).unwrap();
    writeln!(file, "Jens was here. Briefly.").unwrap();
    println!("Saving to tmp directory: {}", tmp.path().to_str().unwrap());

    let app = Application::builder()
        .application_id("se.akernet.logbuddy")
        .build();

    app.connect_activate(build_ui);

    uncompress_recursive(Path::new("test.tar.gz"), tmp.path());
    uncompress_recursive(Path::new("test.tar.gz"), tmp.path());

    app.run();

    drop(file);
    tmp.close().unwrap();
    println!("Goodbye!")
}
