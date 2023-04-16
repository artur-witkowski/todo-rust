use core::fmt;
use std::borrow::BorrowMut;
use std::env;
use std::io::{stdin, stdout};
use std::ops::Add;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

enum ConsoleForegroundColors {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
}

#[derive(PartialEq, Eq)]
enum ConsoleBackgroundColors {
    None = 0,
    Black = 40,
    Red = 41,
    Green = 42,
    Yellow = 43,
    Blue = 44,
    Magenta = 45,
    Cyan = 46,
    White = 47,
}

#[derive(PartialEq, Eq, PartialOrd, Debug)]
enum TaskType {
    Todo,
    Doing,
    Done,
    Rejected,
    NotDefined,
}
impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Copy for TaskType {}

impl Clone for TaskType {
    fn clone(&self) -> TaskType {
        *self
    }
}
impl Ord for TaskType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            TaskType::Todo => match other {
                TaskType::Todo => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Less,
            },
            TaskType::Doing => match other {
                TaskType::Todo => std::cmp::Ordering::Greater,
                TaskType::Doing => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Less,
            },
            TaskType::Done => match other {
                TaskType::Todo => std::cmp::Ordering::Greater,
                TaskType::Doing => std::cmp::Ordering::Greater,
                TaskType::Done => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Less,
            },
            TaskType::Rejected => match other {
                TaskType::Todo => std::cmp::Ordering::Greater,
                TaskType::Doing => std::cmp::Ordering::Greater,
                TaskType::Done => std::cmp::Ordering::Greater,
                TaskType::Rejected => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Less,
            },
            TaskType::NotDefined => match other {
                TaskType::NotDefined => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Greater,
            },
        }
    }
}

// implement for task type option to get next type in order
impl TaskType {
    fn next(&self) -> TaskType {
        match self {
            TaskType::Todo => TaskType::Doing,
            TaskType::Doing => TaskType::Done,
            TaskType::Done => TaskType::Rejected,
            TaskType::Rejected => TaskType::Todo,
            TaskType::NotDefined => TaskType::NotDefined,
        }
    }
}

fn get_color_text(
    color: ConsoleForegroundColors,
    background_color: ConsoleBackgroundColors,
    text: &str,
) -> String {
    if background_color == ConsoleBackgroundColors::None {
        return format!("\x1b[{}m{}\x1b[0m", color as u8, text);
    } else {
        format!(
            "\x1b[{};{}m{}\x1b[0m",
            color as u8, background_color as u8, text
        )
    }
}

fn get_type_from_string(text: &str) -> TaskType {
    if text.starts_with("[+]") {
        TaskType::Doing
    } else if text.starts_with("[X]") {
        TaskType::Done
    } else if text.starts_with("[-]") {
        TaskType::Rejected
    } else if text.starts_with("[ ]") {
        TaskType::Todo
    } else {
        TaskType::NotDefined
    }
}

fn type_to_string(task_type: TaskType) -> String {
    match task_type {
        TaskType::Todo => "[ ]".to_string(),
        TaskType::Doing => "[+]".to_string(),
        TaskType::Done => "[X]".to_string(),
        TaskType::Rejected => "[-]".to_string(),
        TaskType::NotDefined => "[ ]".to_string(),
    }
}

struct Task {
    task_type: TaskType,
    text: String,
}

impl Task {
    fn change_type(&mut self) {
        self.task_type = self.task_type.next();
        self.text
            .replace_range(0..3, &type_to_string(self.task_type));
    }
}

struct TodoList {
    tasks: Vec<Task>,
    is_editing: bool,
}

struct Console {
    cursor_position: (u16, u16),
}

impl Console {
    fn new() -> Self {
        Self {
            cursor_position: (1, 1),
        }
    }

    fn move_cursor(&mut self, direction: Direction) {
        let mut stdout = stdout().into_raw_mode().unwrap();
        match direction {
            Direction::Up => {
                if self.cursor_position.1 > 1 {
                    self.cursor_position.1 -= 1;
                }
            }
            Direction::Down => {
                self.cursor_position.1 += 1;
            }
            Direction::Left => {
                if self.cursor_position.0 > 1 {
                    self.cursor_position.0 -= 1;
                }
            }
            Direction::Right => {
                if self.cursor_position.0 < 10 {
                    self.cursor_position.0 += 1;
                }
            }
        }

        stdout.flush().unwrap();
        write!(
            stdout,
            "{}({}, {})",
            termion::cursor::Goto(25, 25),
            self.cursor_position.0,
            self.cursor_position.1
        )
        .unwrap();
        write!(
            stdout,
            "{}",
            termion::cursor::Goto(self.cursor_position.0, self.cursor_position.1)
        )
        .unwrap();
    }
}

impl TodoList {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            is_editing: false,
        }
    }

    fn load(&mut self, file_path: &str) {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => match File::create(file_path) {
                Ok(_) => File::open(file_path).unwrap(),
                Err(_) => {
                    println!("Could not create file");
                    return;
                }
            },
        };
        let reader = BufReader::new(file);
        self.tasks = Vec::new();
        for line in reader.lines() {
            match line {
                Ok(line) => self.add(line.as_str(), get_type_from_string(line.as_str())),
                Err(_) => continue,
            }
        }
    }

    fn add(&mut self, text: &str, task_type: TaskType) {
        let new_task = Task {
            task_type,
            text: text.to_string(),
        };
        self.tasks.push(new_task);
    }

    fn save(&mut self, file_path: &str) {
        let file = File::create(file_path).unwrap();
        let mut writer = BufWriter::new(file);
        self.tasks.sort_by_key(|task| task.task_type);
        for task in &self.tasks {
            let mut new_line = task.text.to_owned();
            new_line.push_str("\n");
            writer.write(new_line.as_bytes()).unwrap();
        }
    }

    fn print(&mut self, console: &mut Console) {
        let mut stdout = stdout().into_raw_mode().unwrap();
        for (i, task) in self.tasks.iter().enumerate() {
            let mut x_position = 1;
            if self.is_editing && console.cursor_position.1 == (i + 1) as u16 {
                x_position = 3;
            }
            write!(
                stdout,
                "{}{}",
                termion::cursor::Goto(x_position, i as u16 + 1),
                termion::clear::CurrentLine
            )
            .unwrap();
            stdout.flush().unwrap();
            let background_color = if console.cursor_position.1 == (i + 1) as u16 {
                ConsoleBackgroundColors::White
            } else {
                ConsoleBackgroundColors::None
            };
            let mut text = task.text.to_owned();
            if self.is_editing && console.cursor_position.1 == (i + 1) as u16 {
                text.push_str(
                    format!(
                        " (Current: {}, Next: {})",
                        task.task_type,
                        task.task_type.next()
                    )
                    .as_str(),
                );
            }

            if task.task_type == TaskType::Done {
                println!(
                    "{}",
                    get_color_text(
                        ConsoleForegroundColors::Green,
                        background_color,
                        text.as_str()
                    )
                );
            } else if task.task_type == TaskType::Todo {
                println!(
                    "{}",
                    get_color_text(
                        ConsoleForegroundColors::Blue,
                        background_color,
                        text.as_str()
                    )
                );
            } else if task.task_type == TaskType::Doing {
                println!(
                    "{}",
                    get_color_text(
                        ConsoleForegroundColors::Magenta,
                        background_color,
                        text.as_str()
                    )
                );
            } else if task.task_type == TaskType::Rejected {
                println!(
                    "{}",
                    get_color_text(
                        ConsoleForegroundColors::Red,
                        background_color,
                        text.as_str()
                    )
                );
            }
        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("Please provide a path to the file");
        return;
    }

    let file_path = &args[1];
    let mut console = Console::new();

    let mut todo_list = TodoList::new();
    todo_list.load(file_path);
    todo_list.print(&mut console);
    // todo_list.add("Buy milk");
    todo_list.save(file_path);

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Up => {
                if todo_list.is_editing == false {
                    console.move_cursor(Direction::Up);
                }
            }
            Key::Down => {
                if todo_list.is_editing == false {
                    console.move_cursor(Direction::Down)
                }
            }
            Key::Right => {
                todo_list.tasks[(console.cursor_position.1 - 1) as usize].change_type();
                todo_list.is_editing = true;

                todo_list.print(&mut console);
            }
            Key::Left => {
                todo_list.is_editing = false;
                todo_list.save(file_path);
                todo_list.print(&mut console);
            }
            _ => {}
        }

        stdout.flush().unwrap();
        todo_list.print(&mut console);
    }

    todo_list.save(file_path);
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
