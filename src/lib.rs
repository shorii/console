use rustbox;

use std::default::Default;
use std::thread;
use std::time;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use rustbox::{Color, RustBox};
use rustbox::Key;

pub struct Graphic {
    content: Vec<u8>,
    row_length: usize,
    cursor: usize,
}

impl Graphic {
    pub fn new(content: Vec<u8>, row_length: usize) -> Self {
        assert!(content.len() >= row_length);
        match content.len() % row_length {
            0 => Graphic { content, row_length, cursor: 0 },
            _ => panic!("invalid row_length"),
        }
    }
}

impl Iterator for Graphic {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.content.len() {
            return None
        }
        let next_cursor = self.cursor + self.row_length;
        let item = self.content[self.cursor..next_cursor].to_vec();
        self.cursor += self.row_length;
        Some(item)
    }
}

pub trait Keyboard: Send + Sync {
    fn press(&self, key: char);
}

pub struct Console {
    renderer: Arc<Renderer>,
    listener: Arc<EventListener>,
    handles: Vec<thread::JoinHandle<()>>
}

impl Console {
    pub fn new(render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>, keyboard: Box<dyn Keyboard>) -> Self {
        let rustbox = Arc::new(Mutex::new(RustBox::init(Default::default()).unwrap()));

        let renderer = Renderer::new(
            Arc::clone(&rustbox),
            render_bus,
        );
        let renderer = Arc::new(renderer);

        let listener = EventListener::new(Arc::clone(&rustbox), keyboard);
        let listener = Arc::new(listener);

        let handles = Vec::new();
        Console { renderer, listener, handles }
    }

    pub fn run(&mut self) {
        let renderer = Arc::clone(&self.renderer);
        let renderer_handle = thread::spawn(move || {
            loop {
                renderer.run();
            }
        });
        self.handles.push(renderer_handle);

        let listener = Arc::clone(&self.listener);
        let listener_handle = thread::spawn(move || {
            loop {
                listener.run();
            }
        });
        self.handles.push(listener_handle);
    }

    pub fn join(self) {
        for handle in self.handles.into_iter() {
            handle.join();
        }
    }
}

struct EventListener {
    rustbox: Arc<Mutex<RustBox>>,
    keyboard: Box<dyn Keyboard>,
    listener_thread: Option<thread::JoinHandle<()>>,
}

impl EventListener {
    fn new(rustbox: Arc<Mutex<RustBox>>, keyboard: Box<dyn Keyboard>) -> Self{
        let listener_thread: Option<thread::JoinHandle<()>> = None;
        EventListener { rustbox, keyboard, listener_thread }
    }

    fn run(&self) {
        let rustbox = self.rustbox.lock().unwrap();
        match rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char(a) => {
                        self.keyboard.press(a);
                    },
                    _ => {/*do nothing*/}
                }
            },
            _ => panic!("failed to poll event"),
        }
    }
}

struct Renderer {
    rustbox: Arc<Mutex<RustBox>>,
    render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>,
}

impl Renderer {
    fn new(rustbox: Arc<Mutex<RustBox>>, render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>) -> Self{
        Renderer { rustbox, render_bus }
    }

    fn run(&self) {
        let render_bus = self.render_bus.lock().unwrap();
        match render_bus.recv() {
            Ok(graphic) => {
                let rb = self.rustbox.lock().unwrap();
                for (ridx, row) in graphic.enumerate() {
                    for (cidx, column) in row.iter().enumerate() {
                        rb.print(
                            cidx,
                            ridx,
                            rustbox::RB_BOLD,
                            Color::White,
                            match column {
                                0 => Color::Black,
                                _ => Color::White,
                            },
                            " ",
                        );
                    }
                }
                rb.present();
            },
            Err(_) => panic!("cannot draw graphic")
        }
    }
}
