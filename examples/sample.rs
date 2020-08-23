use console::{Keyboard, Graphic, Console};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use rand::prelude::*;

struct Screen {
    gfx: [u8; 192],
    key_bus: Arc<Mutex<mpsc::Receiver<u8>>>,
    render_bus: Arc<Mutex<mpsc::Sender<Graphic>>>
}

impl Screen {
    fn new(key_bus: Arc<Mutex<mpsc::Receiver<u8>>>, render_bus: Arc<Mutex<mpsc::Sender<Graphic>>>) -> Self {
        Screen { gfx: [0; 192], key_bus, render_bus }
    }

    fn run(&self) {
        let mut content = self.gfx.iter().cloned().collect::<Vec<_>>();
        let row_length: usize = 64;

        let key_bus = self.key_bus.lock().unwrap();
        match key_bus.try_recv() {
            Ok(row_num) => {
                let row_num = row_num as usize;
                let offset = row_length * (row_num - 1);
                let begin = offset;
                let end = row_length + offset;
                let idx = rand::thread_rng().gen_range(begin, end);
                content[idx] = 1;
                self.render_bus.lock().unwrap().send(Graphic::new(content, row_length));
            },
            _ => {}
        };
    }
}

struct ScreenRunner {
    inner: Arc<Screen>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ScreenRunner {
    fn new(screen: Screen) -> Self {
        let handle: Option<thread::JoinHandle<()>> = None;
        ScreenRunner { inner: Arc::new(screen), handle }
    }

    fn run(&mut self) {
        let local_self = Arc::clone(&self.inner);
        let handle = thread::spawn(move || {
            loop {
                local_self.run();
            }
        });
        self.handle = Some(handle);
    }

    fn join(self) {
        self.handle.unwrap().join();
    }
}

struct Keypad {
    keyMap: HashMap<char, u8>,
    bus: Arc<Mutex<mpsc::Sender<u8>>>,
}

impl Keypad {
    fn new(bus: Arc<Mutex<mpsc::Sender<u8>>>) -> Self {
        let map: [(char, u8); 3] = [
            ('1', 0x1),
            ('2', 0x2),
            ('3', 0x3),
        ];
        let keyMap = map.iter().cloned().collect::<HashMap<_, _>>();
        Keypad { keyMap, bus }
    }
}

impl Keyboard for Keypad {
    fn press(&self, key: char) {
        match self.keyMap.get(&key) {
            Some(value) => {
                let bus = self.bus.lock().unwrap();
                bus.send(*value);
            },
            None => {/* do nothing */}
        };
    }
}

fn main() {
    let (key_event_sender, key_event_receiver) = mpsc::channel();
    let key_event_sender = Arc::new(Mutex::new(key_event_sender));
    let key_event_receiver = Arc::new(Mutex::new(key_event_receiver));

    let (graphic_sender, graphic_receiver) = mpsc::channel();
    let graphic_sender = Arc::new(Mutex::new(graphic_sender));
    let graphic_receiver = Arc::new(Mutex::new(graphic_receiver));

    let screen = Screen::new(key_event_receiver, graphic_sender);
    let mut screen_runner = ScreenRunner::new(screen);
    screen_runner.run();

    let keypad = Keypad::new(key_event_sender);
    
    let mut console = Console::new(graphic_receiver, Box::new(keypad)).unwrap();
    console.run();
    console.join();
    screen_runner.join();
}
