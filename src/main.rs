const FILE_PATH: &str = "todo.txt";

struct TodoList {
    tasks: Vec<String>,
}

impl TodoList {
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    fn load(&mut self, file_path: &str) {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            self.tasks.push(line.unwrap());
        }
    }

    fn add(&mut self, task: &str) {
        self.tasks.push(task.to_string());
    }

    fn save(&self, file_path: &str) {
        let file = std::fs::File::create(file_path).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        for task in &self.tasks {
            println!(writer, "{}", task).unwrap();
        }
    }
}

fn main() {
    let mut todo_list = TodoList::new();
    todo_list.load(FILE_PATH);
    todo_list.add("Buy milk");
    todo_list.add("Buy eggs");
    todo_list.add("Buy bread");
    todo_list.save(FILE_PATH);
}
