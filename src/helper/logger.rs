use std::{fs::OpenOptions, io::Write};

#[derive(Debug)]
pub struct Logger {
    file: std::fs::File,
}

impl Logger {
    pub fn new(file: &str) -> anyhow::Result<Logger> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(file)?;

        Ok(Logger { file })
    }

    pub fn log(&mut self, message: &str) -> anyhow::Result<()> {
        writeln!(self.file, "{}", message)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! log_message {
    // Variante qui accepte des arguments au format `format!`
    ($($arg:tt)*) => {{
        // Tentative d'accès à l'instance du Logger
        let logger = $crate::INSTANCE.get_or_init(|| {
            $crate::Mutex::new($crate::Logger::new("rusty_editor.log").unwrap())
        });

        // Créer le message formaté
        let message = format!($($arg)*);

        // Utilisation de la référence mutable dans le Mutex
        let mut logger = logger.lock().unwrap();

        // Écrire le message dans le log
        if let Err(e) = logger.log(&message) {
            eprintln!("Error writing to log: {}", e);
        }
    }};
}
