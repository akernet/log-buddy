use std::sync::mpsc::{Sender, Receiver, channel};

enum Command {
    LoadFile{
        path: std::path::PathBuf
    }
}
struct FileHandler {
    sender: Sender<Command>,
    handler: std::thread::JoinHandle<()>
}

impl FileHandler {
    fn new() -> Self {
        let (sender, receiver) = channel();

        let jh = std::thread::spawn(move || {
            loop {
                let command = receiver.recv().unwrap();
                match command {
                    LoadFile => {

                    }
                }
            }
        });

        let this = Self {
            sender: sender,
            handler: jh
        };

        this
    }

    pub fn load_file(&self, path: &std::path::Path) {
        self.sender.send(Command::LoadFile{
            path: path.to_owned()
        }).unwrap();
    }
}