use std::collections::HashMap;

pub struct ArgCollection {
    args: HashMap<String, Option<String>>,
}

impl ArgCollection {
    pub fn new() -> Self {
        let args = Self::get_args();

        let mut collection = HashMap::new();

        let mut cur_arg = None;
        for arg in &args {
            if arg.starts_with('-') {
                if let Some(old_arg) = cur_arg {
                    collection.insert(old_arg, None);
                }

                let arg_key = arg.trim_start_matches('-');
                cur_arg = Some(arg_key.to_string());
            } else {
                if let Some(key) = cur_arg {
                    collection.insert(key.to_string(), Some(arg.clone()));

                    cur_arg = None;
                }
            }
        }

        // process trailing arg
        if let Some(old_arg) = cur_arg {
            collection.insert(old_arg, None);
        }

        Self { args: collection }
    }

    pub fn get_arg_value(&self, arg: &str) -> Option<String> {
        match self.args.get(arg) {
            None => None,
            Some(v) => v.clone(),
        }
    }

    pub fn has_arg(&self, arg: &str) -> bool {
        self.args.contains_key(arg)
    }

    fn get_args() -> Vec<String> {
        std::env::args().collect()
    }
}
