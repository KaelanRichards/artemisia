use std::error::Error;
use thiserror::Error;
use std::fmt::Debug;

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("No more undo steps available")]
    NoUndoAvailable,
    #[error("No more redo steps available")]
    NoRedoAvailable,
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
}

pub trait Command: Send + Sync + Debug {
    fn execute(&self) -> Result<(), Box<dyn Error>>;
    fn undo(&self) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug)]
pub struct History {
    commands: Vec<Box<dyn Command>>,
    current_index: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
        }
    }

    pub fn execute(&mut self, command: Box<dyn Command>) -> Result<(), Box<dyn Error>> {
        // Execute the command
        command.execute()?;

        // If we're not at the end of the history, truncate the redo stack
        if self.current_index < self.commands.len() {
            self.commands.truncate(self.current_index);
        }

        // Add the command to history
        self.commands.push(command);
        self.current_index += 1;

        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), Box<dyn Error>> {
        if self.current_index == 0 {
            return Err(Box::new(HistoryError::NoUndoAvailable));
        }

        self.current_index -= 1;
        self.commands[self.current_index].undo()?;

        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), Box<dyn Error>> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestCommand {
        executed: Arc<AtomicBool>,
        undone: Arc<AtomicBool>,
    }

    impl TestCommand {
        fn new() -> Self {
            Self {
                executed: Arc::new(AtomicBool::new(false)),
                undone: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    impl Command for TestCommand {
        fn execute(&self) -> Result<(), Box<dyn Error>> {
            self.executed.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn undo(&self) -> Result<(), Box<dyn Error>> {
            self.undone.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_history_operations() {
        let mut history = History::new();
        let command = Box::new(TestCommand::new());
        
        // Test execute
        assert!(history.execute(command).is_ok());
        assert!(history.can_undo());
        assert!(!history.can_redo());

        // Test undo
        assert!(history.undo().is_ok());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        // Test redo
        assert!(history.redo().is_ok());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }
} 