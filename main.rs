use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Confirm};
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use json::{object, JsonValue};
use std::fs;

fn load_from_file() -> Result<JsonValue, json::Error>{
    let content = fs::read_to_string("tasks.json").expect("Error reading tasks.json");
    let json = json::parse(&content)?;
    Ok(json)
}

fn save_to_file(json: &mut JsonValue){
    let data = json::stringify(json.clone());
    let mut f = fs::File::create("tasks.json").expect("Unable to a create file.");
    f.write_all(data.as_bytes()).expect("Error writing data.");
}

fn get_new_id(json: &mut JsonValue) -> usize{
    let mut len: usize = 0;
    if let JsonValue::Array(arr) = &json["completed"]{
        len += arr.len();
    }
    if let JsonValue::Array(arr) = &json["tasks"]{
        len += arr.len();
    } else {
        len += 0; // -_-
    }
    len
}

fn append_to_file(json: &mut JsonValue, name: &str, desc: &str){
    let data = object!{
        id: get_new_id(json),
        name: name,
        description: desc,
        time_created: get_unix(),
        completed: false,
        time_completed: 0
    };
    if let JsonValue::Array(ref mut arr) = json["tasks"] {
        arr.push(data);
    }
    save_to_file(json);
}

fn get_unix() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH){
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

fn new_task(json: &mut JsonValue){
    let name: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Task Name")
                        .validate_with(|input: &String| -> Result<(), &str> {
                            if get_id_by_name(json, input.to_string()) == None{
                                Ok(())
                            }
                            else{
                                Err("Task already exists.")
                            }
                        })
                        .interact_text()
                        .unwrap();

    let description: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Task Description")
                        .allow_empty(true)
                        .interact_text()
                        .unwrap();

    append_to_file(json, &name, &description);
}
fn complete_task(json: &mut JsonValue, task_id: usize){
    let mut task_to_complete: Option<JsonValue> = None;

    if let JsonValue::Array(ref mut tasks) = json["tasks"] {
        if let Some(pos) = tasks.iter().position(|t| t["id"].as_usize().unwrap() == task_id) {
            task_to_complete = Some(tasks.remove(pos));
        }
    }



    if let Some(mut task) = task_to_complete {
        task["completed"] = json::JsonValue::Boolean(true);
        task["time_completed"] = get_unix().into();

        if let JsonValue::Array(ref mut completed_tasks) = json["completed"] {
            completed_tasks.push(task);
        }
    }

    save_to_file(json);
}

fn get_id_by_name(json: &mut JsonValue, task: String) -> Option<usize>{
    if let JsonValue::Array(arr) = &json["tasks"] {
        for el in arr {
            if el["name"].to_string() == task{
                return Some(el["id"].as_usize().unwrap());
            }
        }
    }
    None
}

fn select(json: &mut JsonValue){
    let mut items: Vec<String> = Vec::new(); 

    if let JsonValue::Array(arr) = &json["tasks"] {
        for el in arr {
            if el["completed"] == false {
                items.push(el["name"].to_string());
            }
        }
    }
    items.push("New Task".to_string());

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose")
        .items(&items)
        .interact()
        .unwrap();

    if items[selection] == "New Task" {
        new_task(json);
    }
    else {
        let confirmation = Confirm::with_theme(&ColorfulTheme::default())
                            .with_prompt("Mark as completed?")
                            .interact()
                            .unwrap();
        if !confirmation { return; }
        if let Some(task_id) = get_id_by_name(json, items[selection].clone()){
            complete_task(json, task_id);
        }
    }
}

fn main() {
    loop {
        let mut json: JsonValue = match load_from_file() {
            Ok(t) => t,
            Err(e) => {
                println!("Error parsing JSON: {}", e);
                JsonValue::Null
            }
        }; 
        select(&mut json); 
        //println!("{:#}", json);
        //print!("\x1Bc");
        //io::stdout().flush().expect("Failed to flush.");
    }
}
