use crate::keyboard::Keyboard;
use crate::graphic::Graphic;

use rustbox;

use std::default::Default;
use std::thread;
use std::time;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use rustbox::{Color, RustBox};
use rustbox::Key;

pub struct Console {
    renderer: Arc<Renderer>,
    listener: Arc<EventListener>,
    handles: Vec<thread::JoinHandle<()>>
}

impl Console {
    pub fn new(render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>, keyboard: Box<dyn Keyboard>) -> Self {
        let rustbox = Arc::new(RustBox::init(Default::default()).unwrap());

        let renderer = Renderer::new(
            Arc::clone(&rustbox),
            render_bus,
        ); let renderer = Arc::new(renderer);

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
    rustbox: Arc<RustBox>,
    keyboard: Box<dyn Keyboard>,
    listener_thread: Option<thread::JoinHandle<()>>,
}

impl EventListener {
    fn new(rustbox: Arc<RustBox>, keyboard: Box<dyn Keyboard>) -> Self{
        let listener_thread: Option<thread::JoinHandle<()>> = None;
        EventListener { rustbox, keyboard, listener_thread }
    }

    fn run(&self) {
        match self.rustbox.poll_event(false) {
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
    rustbox: Arc<RustBox>,
    render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>,
}

impl Renderer {
    fn new(rustbox: Arc<RustBox>, render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>) -> Self{
        Renderer { rustbox, render_bus }
    }

    fn run(&self) {
        let render_bus = self.render_bus.lock().unwrap();
        match render_bus.try_recv() {
            Ok(graphic) => {
                for (ridx, row) in graphic.enumerate() {
                    for (cidx, column) in row.iter().enumerate() {
                        self.rustbox.print(
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
                self.rustbox.present();
            },
            Err(_) => {/* do nothing */}
        }
    }
}
