use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("No more undo steps available")]
    NoUndoAvailable,
    #[error("No more redo steps available")]
    NoRedoAvailable,
}

pub trait Command: Send + Sync {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn undo(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn description(&self) -> &str;
}

#[derive(Debug)]
pub struct History {
    commands: Vec<Box<dyn Command>>,
    current_index: usize,
    max_steps: usize,
}

impl History {
    pub fn new(max_steps: usize) -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
            max_steps,
        }
    }

    pub fn execute(&mut self, command: Box<dyn Command>) -> Result<(), Box<dyn std::error::Error>> {
        // Execute the command
        command.execute()?;

        // Remove any redoable commands
        self.commands.truncate(self.current_index);

        // Add the new command
        self.commands.push(command);
        self.current_index += 1;

        // Remove oldest commands if we exceed max_steps
        if self.commands.len() > self.max_steps {
            self.commands.remove(0);
            self.current_index -= 1;
        }

        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_index == 0 {
            return Err(Box::new(HistoryError::NoUndoAvailable));
        }

        self.current_index -= 1;
        self.commands[self.current_index].undo()?;
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_index >= self.commands.len() {
            return Err(Box::new(HistoryError::NoRedoAvailable));
        }

        self.commands[self.current_index].execute()?;
        self.current_index += 1;
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_index < self.commands.len()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.current_index = 0;
    }
}

// Example commands
pub struct AddLayerCommand {
    document: Arc<RwLock<super::Document>>,
    layer_name: String,
    layer_id: Option<super::LayerId>,
}

impl AddLayerCommand {
    pub fn new(document: Arc<RwLock<super::Document>>, layer_name: String) -> Self {
        Self {
            document,
            layer_name,
            layer_id: None,
        }
    }
}

impl Command for AddLayerCommand {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut doc = self.document.write();
        let layer = super::Layer::new(self.layer_name.clone());
        let id = doc.add_layer(layer);
        // Store the layer ID for undo
        if let Some(this) = unsafe { (self as *const Self as *mut Self).as_mut() } {
            this.layer_id = Some(id);
        }
        Ok(())
    }

    fn undo(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(id) = &self.layer_id {
            let mut doc = self.document.write();
            doc.remove_layer(id)?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Add Layer"
    }
}

pub struct RemoveLayerCommand {
    document: Arc<RwLock<super::Document>>,
    layer_id: super::LayerId,
    layer_data: Option<super::Layer>,
}

impl RemoveLayerCommand {
    pub fn new(document: Arc<RwLock<super::Document>>, layer_id: super::LayerId) -> Self {
        Self {
            document,
            layer_id,
            layer_data: None,
        }
    }
}

impl Command for RemoveLayerCommand {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut doc = self.document.write();
        // Store the layer data for undo
        if let Some(layer) = doc.get_layer(&self.layer_id) {
            // TODO: Implement Clone for Layer to store its data
        }
        doc.remove_layer(&self.layer_id)?;
        Ok(())
    }

    fn undo(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(layer_data) = &self.layer_data {
            // TODO: Implement restoration of layer data
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Remove Layer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        value: Arc<RwLock<i32>>,
        delta: i32,
    }

    impl Command for TestCommand {
        fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
            let mut value = self.value.write();
            *value += self.delta;
            Ok(())
        }

        fn undo(&self) -> Result<(), Box<dyn std::error::Error>> {
            let mut value = self.value.write();
            *value -= self.delta;
            Ok(())
        }

        fn description(&self) -> &str {
            "Test Command"
        }
    }

    #[test]
    fn test_history() {
        let value = Arc::new(RwLock::new(0));
        let mut history = History::new(10);

        // Execute commands
        history.execute(Box::new(TestCommand {
            value: value.clone(),
            delta: 5,
        })).unwrap();
        assert_eq!(*value.read(), 5);

        history.execute(Box::new(TestCommand {
            value: value.clone(),
            delta: 3,
        })).unwrap();
        assert_eq!(*value.read(), 8);

        // Test undo
        history.undo().unwrap();
        assert_eq!(*value.read(), 5);

        // Test redo
        history.redo().unwrap();
        assert_eq!(*value.read(), 8);
    }
} 