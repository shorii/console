use libc;

use crate::graphic::Graphic;
use crate::keyboard::Keyboard;

use std::default::Default;
use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use signal_hook::{cleanup, flag, SIGTERM};

use rustbox::Key;
use rustbox::{Color, RustBox};

pub struct Console {
    renderer: Arc<Renderer>,
    listener: Arc<EventListener>,
    handles: Vec<thread::JoinHandle<()>>,
    terminated: Arc<AtomicBool>,
}

impl Console {
    pub fn new(
        render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>,
        keyboard: Box<dyn Keyboard>,
        terminated: Arc<AtomicBool>,
    ) -> Result<Self, Error> {
        let rustbox = Arc::new(RustBox::init(Default::default()).unwrap());

        let renderer = Renderer::new(Arc::clone(&rustbox), render_bus);
        let renderer = Arc::new(renderer);

        let listener = EventListener::new(Arc::clone(&rustbox), keyboard);
        let listener = Arc::new(listener);

        let handles = Vec::new();

        flag::register(SIGTERM, Arc::clone(&terminated))?;
        cleanup::register(SIGTERM, vec![SIGTERM])?;

        Ok(Console {
            renderer,
            listener,
            handles,
            terminated,
        })
    }

    fn run_thread<T>(&mut self, service: Arc<T>)
    where
        T: Service + 'static,
    {
        let terminated = Arc::clone(&self.terminated);
        let handle = thread::spawn(move || {
            while !terminated.load(Ordering::Relaxed) {
                service.run();
            }
        });
        self.handles.push(handle);
    }

    pub fn run(&mut self) {
        let renderer = Arc::clone(&self.renderer);
        self.run_thread(renderer);

        let listener = Arc::clone(&self.listener);
        self.run_thread(listener);
    }

    pub fn join(self) {
        for handle in self.handles.into_iter() {
            handle.join().unwrap();
        }
    }
}

trait Service: Send + Sync {
    fn run(&self);
}

struct EventListener {
    rustbox: Arc<RustBox>,
    keyboard: Box<dyn Keyboard>,
}

impl EventListener {
    fn new(rustbox: Arc<RustBox>, keyboard: Box<dyn Keyboard>) -> Self {
        EventListener { rustbox, keyboard }
    }
}

impl Service for EventListener {
    fn run(&self) {
        match self.rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char(a) => {
                        self.keyboard.press(a);
                    }
                    Key::Esc => unsafe {
                        libc::raise(signal_hook::SIGTERM);
                    },
                    _ => { /*do nothing*/ }
                }
            }
            _ => panic!("failed to poll event"),
        }
    }
}

struct Renderer {
    rustbox: Arc<RustBox>,
    render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>,
}

impl Renderer {
    fn new(rustbox: Arc<RustBox>, render_bus: Arc<Mutex<mpsc::Receiver<Graphic>>>) -> Self {
        Renderer {
            rustbox,
            render_bus,
        }
    }
}

impl Service for Renderer {
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
            }
            Err(_) => { /* do nothing */ }
        }
    }
}