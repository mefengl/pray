use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

// Represents a collection of files
#[derive(Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub files: Vec<PathBuf>,
    pub num_files: usize,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

// Enum representing which pane is currently focused
pub enum FocusedPane {
    FilesPane,
    CollectionsPane,
    SelectedFilesPane,
}

// The main application state
pub struct App {
    // Current directory path
    pub current_dir: PathBuf,
    // List of directory entries in the current directory
    pub directory_entries: Vec<PathBuf>,
    // Index of the selected item in the files pane
    pub selected_file_index: usize,
    // Index of the selected collection
    pub selected_collection_index: usize,
    // Index of the selected file in the selected collection
    pub selected_file_in_collection_index: usize,
    // Store selected items in the current directory
    pub selected_items: HashSet<PathBuf>,
    // Base directory for relative paths
    pub base_dir: PathBuf,
    // Stack to keep track of navigation and cursor positions
    pub navigation_stack: Vec<(PathBuf, usize)>,
    // Message to display in the footer
    pub footer_message: Option<String>,
    // Counter to keep track of message display duration
    pub message_counter: u8,
    // Flag for select all state in files pane
    pub all_selected: bool,
    // List of collections
    pub collections: Vec<Collection>,
    // Path to the collections file
    pub collections_file: PathBuf,
    // Focused pane
    pub focused_pane: FocusedPane,
    // Flag to show help screen
    pub show_help: bool,
    // Renaming state
    pub renaming_collection: bool,
    pub new_collection_name: String,
}

impl App {
    // Create a new `App` instance.
    pub fn new() -> App {
        // Start at the current working directory
        let current_dir = std::env::current_dir().unwrap();
        let directory_entries = Self::read_directory(&current_dir);

        // Set the base directory to the current directory
        let base_dir = current_dir.clone();

        // Set the path to the collections file in the data local directory
        let project_dirs = ProjectDirs::from("", "", "pray").unwrap();
        let data_local_dir = project_dirs.data_local_dir();
        fs::create_dir_all(data_local_dir).unwrap();
        let collections_file = data_local_dir.join("collections.json");

        // Attempt to read the collections from the file
        let collections = if collections_file.exists() {
            let file = fs::File::open(&collections_file).unwrap();
            serde_json::from_reader(file).unwrap_or_else(|_| vec![])
        } else {
            vec![]
        };

        App {
            base_dir,
            current_dir: current_dir.clone(),
            directory_entries,
            selected_file_index: 0,
            selected_collection_index: 0,
            selected_file_in_collection_index: 0,
            selected_items: HashSet::new(),
            navigation_stack: vec![],
            footer_message: None,
            message_counter: 0,
            all_selected: false,
            collections,
            collections_file,
            focused_pane: FocusedPane::FilesPane,
            show_help: false,
            renaming_collection: false,
            new_collection_name: String::new(),
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
        let selected_path = &self.directory_entries[self.selected_file_index];
        if selected_path.is_dir() {
            // Push current state onto the navigation stack
            self.navigation_stack
                .push((self.current_dir.clone(), self.selected_file_index));
            self.current_dir = selected_path.clone();
            self.directory_entries = Self::read_directory(&self.current_dir);
            self.selected_file_index = 0;
        }
    }

    // Go back to parent directory
    pub fn go_back(&mut self) {
        if let Some((previous_dir, previous_index)) = self.navigation_stack.pop() {
            self.current_dir = previous_dir;
            self.directory_entries = Self::read_directory(&self.current_dir);
            self.selected_file_index = previous_index;
        }
    }

    // Toggle selection of the current item
    pub fn toggle_selection(&mut self) {
        if let Some(selected_path) = self.directory_entries.get(self.selected_file_index) {
            if self.selected_items.contains(selected_path) {
                self.selected_items.remove(selected_path);
            } else {
                self.selected_items.insert(selected_path.clone());
            }
        }
    }

    // Check if all items in current directory are selected
    fn is_current_dir_all_selected(&self) -> bool {
        self.directory_entries
            .iter()
            .all(|entry| self.selected_items.contains(entry))
    }

    // Select or deselect all items in current directory only
    pub fn toggle_select_all(&mut self) {
        let current_all_selected = self.is_current_dir_all_selected();

        // Remove only current directory items from selection
        self.selected_items
            .retain(|item| !self.directory_entries.contains(item));

        if !current_all_selected {
            // Add all current directory items to selection
            self.selected_items
                .extend(self.directory_entries.iter().cloned());
        }

        self.all_selected = !current_all_selected;
    }

    fn get_all_files_in_dir(&self, dir: &PathBuf) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                } else if path.is_dir() {
                    files.extend(self.get_all_files_in_dir(&path));
                }
            }
        }
        files
    }

    pub fn copy_selected_items_to_clipboard(&mut self) {
        use clipboard::{ClipboardContext, ClipboardProvider};
        use std::io::Read;

        let mut output = String::new();
        let mut all_files = Vec::new();

        // Collect all files, including those in selected directories
        for item in &self.selected_items {
            if item.is_file() {
                all_files.push(item.clone());
            } else if item.is_dir() {
                all_files.extend(self.get_all_files_in_dir(item));
            }
        }

        for item in &all_files {
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

        // Copy to clipboard
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.set_contents(output.clone()).unwrap();

        // Display success message in footer
        self.footer_message = Some("Copied to clipboard!".to_string());
        self.message_counter = 5; // Display for 5 cycles

        // Create new collection and add to collections
        let collection_name = format!("Collection {}", self.collections.len() + 1);

        let collection = Collection {
            name: collection_name,
            files: all_files.clone(),
            num_files: all_files.len(),
            timestamp: chrono::Local::now(),
        };

        self.collections.push(collection);
        self.save_collections();

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

    // Remove the selected collection
    pub fn remove_selected_collection(&mut self) {
        if self.collections.is_empty() {
            return;
        }

        self.collections.remove(self.selected_collection_index);
        if self.selected_collection_index >= self.collections.len()
            && self.selected_collection_index > 0
        {
            self.selected_collection_index -= 1;
        }
        self.save_collections();
    }

    // Copy files from the selected collection to clipboard
    pub fn copy_selected_collection_to_clipboard(&mut self) {
        use clipboard::{ClipboardContext, ClipboardProvider};
        use std::io::Read;

        if self.collections.is_empty() {
            return;
        }

        let collection = &self.collections[self.selected_collection_index];

        let mut output = String::new();

        for item in &collection.files {
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
        self.footer_message = Some("Collection copied to clipboard!".to_string());
        self.message_counter = 5; // Display for 5 cycles
    }

    // Unselect a file from the selected collection
    pub fn unselect_file_from_collection(&mut self) {
        if self.collections.is_empty() {
            return;
        }
        let collection = &mut self.collections[self.selected_collection_index];
        if self.selected_file_in_collection_index < collection.files.len() {
            collection
                .files
                .remove(self.selected_file_in_collection_index);
            collection.num_files = collection.files.len();

            // Adjust index if necessary
            if self.selected_file_in_collection_index >= collection.files.len()
                && self.selected_file_in_collection_index > 0
            {
                self.selected_file_in_collection_index -= 1;
            }

            // Move this outside the mutable borrow of collection
            self.save_collections();
        }
    }

    // Save collections to the collections file
    fn save_collections(&self) {
        let file = fs::File::create(&self.collections_file).unwrap();
        serde_json::to_writer(file, &self.collections).unwrap();
    }

    // Start renaming a collection
    pub fn start_rename(&mut self) {
        if self.collections.is_empty() {
            return;
        }
        self.renaming_collection = true;
        self.new_collection_name = self.collections[self.selected_collection_index]
            .name
            .clone();
    }

    // Confirm the rename operation
    pub fn confirm_rename(&mut self) {
        if self.collections.is_empty() || !self.renaming_collection {
            return;
        }
        self.collections[self.selected_collection_index].name = self.new_collection_name.clone();
        self.save_collections();
        self.renaming_collection = false;
        self.new_collection_name.clear();

        // Display success message
        self.footer_message = Some("Collection renamed!".to_string());
        self.message_counter = 5; // Display for 5 cycles
    }

    // Cancel the rename operation
    pub fn cancel_rename(&mut self) {
        if self.renaming_collection {
            self.renaming_collection = false;
            self.new_collection_name.clear();

            // Display cancellation message
            self.footer_message = Some("Rename canceled.".to_string());
            self.message_counter = 5; // Display for 5 cycles
        }
    }
}
