use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::{self, prelude::*};
use std::path::Path;
use std::{
    env,
    time::{Duration, SystemTime},
};

#[derive(Copy, Clone, Debug, PartialEq)]
enum Status {
    Todo,
    InProgress,
    Done,
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match &self {
            Status::Todo => "todo",
            Status::InProgress => "in-progress",
            Status::Done => "done",
        };
        f.write_str(display)
    }
}

#[derive(Debug)]
struct Task {
    id: u32,
    description: String,
    status: Status,
    // For sake of writing/parsing file format and only using the std library, I am going to use the number of seconds since SystemTime::UNIX_EPOCH
    created_at: SystemTime,
    updated_at: SystemTime,
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let created_at_since_epoch = {
            let mut time_secs = 0;
            if let Ok(duration) = self.created_at.duration_since(SystemTime::UNIX_EPOCH) {
                time_secs = duration.as_secs();
            };
            time_secs
        };
        let updated_at_since_epoch = {
            let mut time_secs = 0;
            if let Ok(duration) = self.updated_at.duration_since(SystemTime::UNIX_EPOCH) {
                time_secs = duration.as_secs();
            };
            time_secs
        };
        write!(f, "\n\"id\":{0},\n\"description\": \"{1}\",\n\"status\": \"{2}\",\n\"created_at\": {3},\n\"updated_at\": {4}\n", 
            self.id, self.description, self.status, created_at_since_epoch, updated_at_since_epoch)
    }
}

#[derive(Default)]
struct TaskHandler {
    tasks: HashMap<u32, Task>,
    updated: bool,
}

impl std::fmt::Display for TaskHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut json = String::new();
        let len = self.tasks.len();
        for (i, (id, task)) in self.tasks.iter().enumerate() {
            json.push_str(format!("\n\"{}\" : {{ {} }}", id, task).as_str());
            if i < len - 1 {
                json.push_str(",\n");
            } else {
                json.push('\n');
            }
        }
        write!(f, "{}", json)
    }
}

impl TaskHandler {
    // Adds a new task with the first available Id
    fn add(&mut self, description: String) {
        let created_at = SystemTime::now();
        let mut id = 0u32;
        while self.tasks.contains_key(&id) {
            id += 1;
        }
        let task = Task {
            id,
            description,
            status: Status::Todo,
            created_at,
            updated_at: created_at,
        };
        self.tasks.insert(id, task);
        println!("Task added successfully (ID: {})", id);
        self.updated = true;
    }
    // Updates a task with a given Id
    fn update(&mut self, id: u32, description: String) {
        let updated_at = SystemTime::now();
        if let Some(task) = self.tasks.get(&id) {
            let updated_task = Task {
                id,
                description,
                status: task.status,
                created_at: task.created_at,
                updated_at,
            };
            self.tasks.insert(id, updated_task);
            println!("Task updated successfully (ID: {})", id);
            self.updated = true;
        } else {
            println!("Task not available, please create new task with ID: {}", id);
        }
    }
    // Updates a task with a given Id to in_progress
    fn mark_in_progress(&mut self, id: u32) {
        let updated_at = SystemTime::now();
        if let Some(task) = self.tasks.get(&id) {
            let updated_task = Task {
                id,
                description: task.description.clone(),
                status: Status::InProgress,
                created_at: task.created_at,
                updated_at,
            };
            self.tasks.insert(id, updated_task);
            println!("Task updated successfully (ID: {})", id);
            self.updated = true;
        } else {
            println!("Task not available, please create new task with ID: {}", id);
        }
    }
    // Updates a task with a given Id to done
    fn mark_done(&mut self, id: u32) {
        let updated_at = SystemTime::now();
        if let Some(task) = self.tasks.get(&id) {
            let updated_task = Task {
                id,
                description: task.description.clone(),
                status: Status::Done,
                created_at: task.created_at,
                updated_at,
            };
            self.tasks.insert(id, updated_task);
            println!("Task updated successfully (ID: {})", id);
            self.updated = true;
        } else {
            println!("Task not available, please create new task with ID: {}", id);
        }
    }
    // List the current tasks and can pass an optional filter on todo, in-progress, and done
    fn list(&self, filter: Option<Status>) {
        println!("{:<6}{:<30}{:<10}", "id", "description", "status");
        println!("{:-<46}", "-");
        for task in self.tasks.values() {
            if let Some(status) = filter
                && task.status != status
            {
                continue;
            }
            println!("{:<6}{:<30}{:<10}", task.id, task.description, task.status);
        }
    }
    // Deletes a task if it does exist with a given id
    fn delete(&mut self, id: u32) {
        if self.tasks.remove(&id).is_some() {
            println!("Task deleted successfully (ID: {})", id);
            self.updated = true;
        } else {
            println!("Task failed to delete or does not exist (ID: {})", id);
        }
    }
    // Everytime the command is run, the tasks.json file is parsed to provide the latest
    fn query_task_file() -> Self {
        let mut tasks = HashMap::new();

        if let Ok(data) = read_to_string("tasks.json") {
            let data = data.replace("\"", "");
            let parts: Vec<&str> = data
                .split(&['{', '}'][..])
                .map(|p| p.trim().trim_end_matches(':').trim())
                .filter(|p| !p.is_empty())
                .collect();
            let mut id = 0;
            for (i, id_task) in parts.iter().skip(1).enumerate() {
                if i % 2 == 1 {
                    let task_parts: Vec<&str> = id_task
                        .split(',')
                        .map(|p| p.trim().split(':').collect::<Vec<_>>()[1])
                        .collect();
                    let inner_id = task_parts[0]
                        .trim()
                        .parse::<u32>()
                        .expect("failed to convert id from json");
                    if id != inner_id {
                        println!("id and inner id don't match");
                    }
                    let description = task_parts[1]
                        .trim()
                        .parse::<String>()
                        .expect("failed to convert description from json");
                    let status = task_parts[2]
                        .trim()
                        .parse::<String>()
                        .map(|s| match s.as_str() {
                            "todo" => Status::Todo,
                            "in-progress" => Status::InProgress,
                            "done" => Status::Done,
                            _ => Status::Todo,
                        })
                        .expect("failed to convert status from json");
                    let created_at = task_parts[3]
                        .trim()
                        .parse::<u64>()
                        .map(|t| SystemTime::UNIX_EPOCH + Duration::from_secs(t))
                        .expect("failed to convert created_at from json");
                    let updated_at = task_parts[4]
                        .trim()
                        .parse::<u64>()
                        .map(|t| SystemTime::UNIX_EPOCH + Duration::from_secs(t))
                        .expect("failed to convert updated_at from json");

                    let task = Task {
                        id,
                        description,
                        status,
                        created_at,
                        updated_at,
                    };
                    tasks.insert(id, task);
                } else {
                    id = id_task
                        .trim_matches(',')
                        .trim()
                        .parse::<u32>()
                        .expect("failed to convert id from key");
                }
            }
        }

        TaskHandler {
            tasks,
            updated: false,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut tasks = TaskHandler::query_task_file();

    let args_passed = args.len() - 1;

    if args_passed > 0 {
        // map the required number of arguments with the available
        // this part could be done in different ways but this was a quick one to setup with the
        // small number of arguments
        // failure here are related to the parsing argument process before handling it to the
        // function
        match (args[1].as_str(), args_passed) {
            ("add", 2) => {
                if let Some(task_description) = args.get(2) {
                    tasks.add(task_description.to_string());
                } else {
                    println!("failed to parse task");
                }
            }
            ("update", 3) => {
                if let Some(id) = args.get(2) {
                    if let Some(updated_task_description) = args.get(3) {
                        let id = id.parse::<u32>().expect("id must be a number");
                        tasks.update(id, updated_task_description.to_string());
                    } else {
                        println!("failed to parse updated task");
                    }
                } else {
                    println!("failed to parse id");
                }
            }
            ("delete", 2) => {
                if let Some(id) = args.get(2) {
                    let id = id.parse::<u32>().expect("id must be a number");
                    tasks.delete(id);
                } else {
                    println!("failed to parse id");
                }
            }
            ("list", 1..=2) => {
                // Not the best work to achieve this but might come back to it later
                let filter = match args.get(2) {
                    Some(status) => match status.as_str() {
                        "done" => Some(Status::Done),
                        "todo" => Some(Status::Todo),
                        "in-progress" => Some(Status::InProgress),
                        _ => {
                            println!("Not a valid status for task");
                            None
                        }
                    },
                    None => None,
                };

                tasks.list(filter);
            }
            ("mark-in-progress", 2) => {
                if let Some(id) = args.get(2) {
                    let id = id.parse::<u32>().expect("id must be a number");
                    tasks.mark_in_progress(id);
                } else {
                    println!("failed to parse id");
                }
            }
            ("mark-done", 2) => {
                if let Some(id) = args.get(2) {
                    let id = id.parse::<u32>().expect("id must be a number");
                    tasks.mark_done(id);
                } else {
                    println!("failed to parse id");
                }
            }
            (_, _) => {
                println!("Invalid argument input");
            }
        }
    } else {
        println!("Invalid amount of arguments, must provide one argument for action");
    }
    if tasks.updated {
        // Print to file
        // Relies on the fmt of the TaskHandler and Tasks to produce valid json
        let mut file = File::create("tasks.json").expect("failed to create file");
        let _ = writeln!(file, "{{\n \"tasks\": {{ {} }} \n }}", tasks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // left out due to being stdout but could do integration tests on:
    // list
    // list done
    // list in-progress
    // list todo
    //
    // also ignoring created_by and updated_by since SystemTime is non-deterministic

    #[test]
    fn add_task() {
        let mut task_handler = TaskHandler::default();
        task_handler.add(String::from("task_a"));
        let made_task = task_handler.tasks.get(&0).unwrap();
        assert_eq!(0, made_task.id);
        assert_eq!("task_a", made_task.description);
        assert_eq!(Status::Todo, made_task.status);
    }
    #[test]
    fn update_task() {
        let mut task_handler = TaskHandler::default();
        task_handler.add(String::from("task_a"));
        task_handler.update(0, String::from("task_b"));
        let made_task = task_handler.tasks.get(&0).unwrap();
        assert_eq!(0, made_task.id);
        assert_eq!("task_b", made_task.description);
        assert_eq!(Status::Todo, made_task.status);
    }
    #[test]
    fn delete_task() {
        let mut task_handler = TaskHandler::default();
        task_handler.add(String::from("task_a"));
        task_handler.delete(0);
        assert!(task_handler.tasks.is_empty());
    }
    #[test]
    fn mark_task_in_progress() {
        let mut task_handler = TaskHandler::default();
        task_handler.add(String::from("task_a"));
        task_handler.mark_in_progress(0);
        let made_task = task_handler.tasks.get(&0).unwrap();
        assert_eq!(0, made_task.id);
        assert_eq!("task_a", made_task.description);
        assert_eq!(Status::InProgress, made_task.status);
    }
    #[test]
    fn mark_task_done() {
        let mut task_handler = TaskHandler::default();
        task_handler.add(String::from("task_a"));
        task_handler.mark_done(0);
        let made_task = task_handler.tasks.get(&0).unwrap();
        assert_eq!(0, made_task.id);
        assert_eq!("task_a", made_task.description);
        assert_eq!(Status::Done, made_task.status);
    }
}
