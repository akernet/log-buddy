mod sidebar_file_list;

use gtk::prelude::*;
use gtk::*;
use gdk;
use glib::clone;
use std::rc::Rc;
use tempfile::tempdir;

use compress_tools::*;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::io::Read;
use std::io::prelude::*;
use std::io::SeekFrom;

use walkdir::WalkDir;

use log::{info, debug};
use simple_logger::SimpleLogger;

use crate::sidebar_file_list::SidebarFileList;

struct AddFileEvent {
    name: String
}

enum EventType {
    AddFileEvent()
}

#[derive(Clone)]
struct Main {
    sender: Rc<gtk::glib::Sender<EventType>>,
    receiver: Rc<gtk::glib::Receiver<EventType>>,
    text_buffer: Rc<TextBuffer>,
    text_view: Rc<TextView>,
    tmp: Rc<PathBuf>
}

impl Main {
    fn init(app: &Application, tmp: PathBuf) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Log Buddy")
            .default_height(600)
            .default_width(600)
            .build();

        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let text_buffer = TextBuffer::builder()
            .build();

        let text_view = TextView::builder()
            .vexpand(true)
            .hexpand(true)
            .monospace(true)
            .buffer(&text_buffer)
            .build();

        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
            .min_content_width(360)
            .min_content_height(300)
            .child(&text_view)
            .kinetic_scrolling(true)
            .vexpand(true)
            .build();

        let minimap = DrawingArea::builder()
            .vexpand(true)
            .width_request(30)
            .build();

        let text_view_with_minimap = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();

        let sidebar = Notebook::builder()
            .build();

        let sidebar_log_file_browser_list = ListBox::builder()
            .build();

        for number in 0..=100 {
            let label = number.to_string();
            let file_entry = Self::get_file_list_entry(&label);
            sidebar_log_file_browser_list.append(&file_entry);
        }

        let sidebar_log_file_browser = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
            .child(&sidebar_log_file_browser_list)
            .build();
        let sidebar_log_file_browser_label = Label::builder()
            .label("Files")
            .build();

        let b1 = Button::builder().label("test1").build();
        let b2 = Button::builder().label("test2").build();

        let sidebar_log_highlight_browser = Paned::builder()
            .orientation(Orientation::Vertical)
            .start_child(&b1)
            .end_child(&b2)
            .build();
        let sidebar_log_highlight_browser_label = Label::builder()
            .label("Highlights")
            .build();

        let sidebar_settings = Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        let sidebar_settings_label = Label::builder()
            .label("Settings")
            .build();

        sidebar.append_page_menu(&sidebar_log_file_browser, Some(&sidebar_log_file_browser_label), Some(&sidebar_log_file_browser_label));
        sidebar.append_page_menu(&sidebar_log_highlight_browser, Some(&sidebar_log_highlight_browser_label), Some(&sidebar_log_highlight_browser_label));
        sidebar.append_page_menu(&sidebar_settings, Some(&sidebar_settings_label), Some(&sidebar_settings_label));

        let main_split_panes = Paned::builder()
            .start_child(&sidebar)
            .end_child(&text_view_with_minimap)
            .position(300)
            .build();

        let s = Self {
            sender: Rc::new(sender),
            receiver: Rc::new(receiver),
            text_buffer: Rc::new(text_buffer),
            text_view: Rc::new(text_view),
            tmp: Rc::new(tmp)
        };

        let drop_target = s.get_drop_target_controller();
        window.add_controller(&drop_target);
        s.text_view.add_controller(&drop_target);

        text_view_with_minimap.append(&scrolled_window);
        text_view_with_minimap.append(&minimap);

        let tag = TextTag::builder()
            .weight(800)
            .build();

        s.text_buffer.tag_table().add(&tag);

        let text_buffer = s.text_buffer.clone();
        let text_view = s.text_view.clone();
        /*
        button1.connect_clicked(clone!(@weak text_buffer => move |_| {
            let start = text_buffer.iter_at_line(0).unwrap();
            let end = text_buffer.iter_at_line(5).unwrap();
            text_buffer.remove_all_tags(&text_buffer.iter_at_offset(0), &text_buffer.iter_at_offset(-1));
            text_buffer.apply_tag(&tag, &start, &end);

            // Initial logic to find visible lines, more work needed
            let top_display_coordinate = text_view.window_to_buffer_coords(TextWindowType::Widget, 0, 0).0;
            let bottom_display_coordinate = text_view.window_to_buffer_coords(TextWindowType::Widget, 0, text_view.height()).0;

            let top_line = text_view.iter_at_position(0, top_display_coordinate).unwrap().0.line();
            let bottom_line = text_view.iter_at_position(0, bottom_display_coordinate).unwrap().0.line();
            println!("{} {}", top_line, bottom_line);

            println!("Applied tag! {} {}", start.line(), end.line());
        }));
        */

        window.set_child(Some(&main_split_panes));

        window.present();

        s
    }

    fn get_drop_target_controller(&self) -> DropTarget {
        // TODO: implement multiple file drop
        let drop_target = DropTarget::new(glib::Type::STRING, gdk::DragAction::COPY);
        drop_target.set_types(&[gtk::gio::File::static_type()]);

        drop_target.connect_drop(clone!(@strong self as this => move |_, v, _, _| {
            // Set the text view back to targetable
            this.text_view.as_ref().set_can_target(true);

            let file = v.get::<gtk::gio::File>();
            match &file {
                Ok(file) => {
                    match file.path() {
                        Some(path) => {
                            println!("File! {:?}", file.path());
                            this.load_file(path);
                            true
                        },
                        _ => {
                            eprintln!("Couldn't get file path");
                            false
                        }
                    }
                },
                _ => {
                    eprintln!("Invalid drop");
                    false
                }
            }
        }));

        drop_target.connect_enter(clone!(@strong self as this => move |a, b, c| {
            // Disable targeting for the text view since we don't want it to eat
            // file drops. There is probably a better way to do this
            this.text_view.as_ref().set_can_target(false);
            gdk::DragAction::COPY
        }));

        drop_target.connect_leave(clone!(@strong self as this => move |a| {
            // Set the text view back to targetable
            this.text_view.as_ref().set_can_target(true);
        }));

        drop_target
    }

    fn uncompress_recursive(path: &Path) -> Vec<PathBuf> {
        // TODO: figure out what this needs to be
        const BUFFER_LEN: usize = 1000;

        let mut source = std::fs::File::open(path).unwrap();

        // Read some bytes and try to figure out file type
        let mut buffer: [u8; BUFFER_LEN] = [0; BUFFER_LEN];
        let bytes_read = source.read(&mut buffer).unwrap();
        let file_kind = infer::Infer::new().get(&buffer[0..bytes_read]);

        let mut result: Vec<PathBuf> = Vec::new();
        match file_kind {
            Some(kind) => {
                match kind.matcher_type() {
                    infer::MatcherType::Archive => {
                        let dirname = path.parent().unwrap();
                        let new_directory_name = format!("{}_uncompressed", path.file_name().unwrap().to_str().unwrap());
                        let dest = Path::new(dirname).join(new_directory_name);

                        debug!("Unpacking archive {:?} to new directory {:?}", path, dest);
                        // Seek back to beginning so uncompress_archive starts reading from the start
                        source.seek(SeekFrom::Start(0)).unwrap();
                        uncompress_archive(&mut source, &dest, Ownership::Ignore).unwrap();

                        // Recurse down
                        WalkDir::new(dest)
                            .into_iter()
                            .filter_map(|e| e.ok())
                            .filter(|f| f.metadata().unwrap().is_file())
                            .map(|file| Self::uncompress_recursive(file.path()))
                            .for_each(|file_vector| result.extend(file_vector));

                    }
                    _ => debug!("Other type for path {:?}", &path)
                }
            }
            None => debug!("Unable to get file type for {:?}", &path)
        };

        if result.is_empty() {
            result.push(PathBuf::from(path));
        }
        result
    }


    // Function that takes a path and recursively finds all log lines,
    // unpacking archives when needed.
    fn load_file(&self, path: std::path::PathBuf) {
        let sender = self.sender.as_ref().clone();
        let tmp_dir = (*self.tmp).clone();
        std::thread::spawn(move || {
            let dest = tmp_dir.as_path().join(path.clone().file_name().unwrap());
            info!("Copying {:?} to {:?}", path, &dest);
            std::fs::copy(path.clone(), &dest).unwrap();
            let res = Self::uncompress_recursive(&dest.as_path());
            info!("Got file list {:?}", res);
        });
    }

    fn get_file_list_entry(text: &str) -> Box {
        let check = CheckButton::builder()
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .active(true)
            .build();
        let label = Label::builder()
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .label(text)
            .build();
        let boxx = Box::builder()
            .build();
        boxx.append(&check);
        boxx.append(&label);
        boxx
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let tmp = tempdir().unwrap();
    info!(target: "logbuddy", "Initiated tmp directory at {}", tmp.path().to_str().unwrap());


    let app =  Application::builder()
        .application_id("se.akernet.logbuddy")
        .build();

    let tmpbuf = PathBuf::from(&tmp.path());
    app.connect_activate(move |app| {
        Main::init(app, tmpbuf.clone());
    });

    app.run();


    info!(target: "logbuddy", "Cleaning up tmp directory at {}", tmp.path().to_str().unwrap());
    tmp.close().unwrap();
    info!(target: "logbuddy", "Bye!");
}
