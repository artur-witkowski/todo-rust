use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

const FILE_PATH: &str = "./todo.txt";

struct TodoList {
    tasks: Vec<String>,
}

impl TodoList {
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    fn load(&mut self, file_path: &str) {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => File::create(file_path).unwrap(),
        };
        let reader = BufReader::new(file);

        for line in reader.lines() {
            self.tasks.push(match line {
                Ok(line) => line,
                Err(_) => continue,
            });
        }
    }

    fn add(&mut self, task: &str) {
        self.tasks.push(task.to_string());
    }

    fn save(&self, file_path: &str) {
        let file = File::create(file_path).unwrap();
        let mut writer = BufWriter::new(file);
        for task in &self.tasks {
            let mut new_line = task.to_owned();
            new_line.push_str("\n");
            writer.write(new_line.as_bytes()).unwrap();
        }
    }

    fn read(&self) {
        for task in &self.tasks {
            println!("{}", task);
        }
    }
}

fn main() {
    let mut todo_list = TodoList::new();
    todo_list.load(FILE_PATH);
    todo_list.read();
    todo_list.add("Buy milk");
    todo_list.add("Buy eggs");
    todo_list.add("Buy bread");
    todo_list.save(FILE_PATH);
}
