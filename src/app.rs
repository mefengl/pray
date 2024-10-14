use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub enum CurrentScreen {
    Main,
    Help,
}

pub struct App {
    // Current directory path
    pub current_dir: PathBuf,
    // List of directory entries in the current directory
    pub directory_entries: Vec<PathBuf>,
    // Index of the selected item
    pub selected_index: usize,
    pub current_screen: CurrentScreen,
    // Store selected items
    pub selected_items: HashSet<PathBuf>,
    // Base directory for relative paths
    pub base_dir: PathBuf,
    // Stack to keep track of navigation and cursor positions
    pub navigation_stack: Vec<(PathBuf, usize)>,
    // Message to display in the footer
    pub footer_message: Option<String>,
    // Counter to keep track of message display duration
    pub message_counter: u8,
    // Flag for select all state
    pub all_selected: bool,
}

impl App {
    pub fn new() -> App {
        // Start at the current working directory
        let current_dir = std::env::current_dir().unwrap();
        let directory_entries = Self::read_directory(&current_dir);

        App {
            base_dir: current_dir.clone(),
            current_dir: current_dir.clone(),
            directory_entries,
            selected_index: 0,
            current_screen: CurrentScreen::Main,
            selected_items: HashSet::new(),
            navigation_stack: vec![],
            footer_message: None,
            message_counter: 0,
            all_selected: false,
        }
    }

    // Read the directory entries
    fn read_directory(path: &PathBuf) -> Vec<PathBuf> {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .unwrap()
            .filter_map(|res| res.ok().map(|e| e.path()))
            .collect();
        entries.sort();
        entries
    }

    // Enter a directory
    pub fn enter_directory(&mut self) {
        if self.directory_entries.is_empty() {
            return;
        }
        let selected_path = &self.directory_entries[self.selected_index];
        if selected_path.is_dir() {
            // Push current state onto the navigation stack
            self.navigation_stack
                .push((self.current_dir.clone(), self.selected_index));
            self.current_dir = selected_path.clone();
            self.directory_entries = Self::read_directory(&self.current_dir);
            self.selected_index = 0;
        }
    }

    // Go back to parent directory
    pub fn go_back(&mut self) {
        if let Some((previous_dir, previous_index)) = self.navigation_stack.pop() {
            self.current_dir = previous_dir;
            self.directory_entries = Self::read_directory(&self.current_dir);
            self.selected_index = previous_index;
        }
    }

    // Toggle selection of the current item
    pub fn toggle_selection(&mut self) {
        if let Some(selected_path) = self.directory_entries.get(self.selected_index) {
            if self.selected_items.contains(selected_path) {
                self.selected_items.remove(selected_path);
            } else {
                self.selected_items.insert(selected_path.clone());
            }
        }
    }

    // Select or deselect all items
    pub fn toggle_select_all(&mut self) {
        if self.all_selected {
            self.selected_items.clear();
            self.all_selected = false;
        } else {
            self.selected_items
                .extend(self.directory_entries.iter().cloned());
            self.all_selected = true;
        }
    }

    // Generate output and copy to clipboard
    pub fn copy_selected_items_to_clipboard(&mut self) {
        use clipboard::{ClipboardContext, ClipboardProvider};
        use std::io::Read;

        let mut output = String::new();

        for item in &self.selected_items {
            if item.is_file() {
                if let Ok(mut file) = fs::File::open(item) {
                    let mut contents = String::new();
                    if let Ok(_) = file.read_to_string(&mut contents) {
                        let relative_path = item.strip_prefix(&self.base_dir).unwrap_or(item);
                        output.push_str(&format!("------ {} ------\n", relative_path.display()));
                        output.push_str("``````\n");
                        output.push_str(&contents);
                        output.push_str("\n``````\n");
                    }
                }
            }
        }

        // Copy to clipboard
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.set_contents(output).unwrap();

        // Display success message in footer
        self.footer_message = Some("Copied to clipboard!".to_string());
        self.message_counter = 5; // Display for 5 cycles

        // Reset selected items and all_selected flag
        self.selected_items.clear();
        self.all_selected = false;
    }

    // Decrement message counter
    pub fn decrement_message_counter(&mut self) {
        if self.message_counter > 0 {
            self.message_counter -= 1;
            if self.message_counter == 0 {
                self.footer_message = None;
            }
        }
    }
}
