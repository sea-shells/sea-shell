use std::format as f;

pub mod commands;
mod state;
pub use state::State;

#[derive(Debug)]
pub struct Pirs {
  pub state: State,
  exit_handler: fn(i32),
  pub logger: Box<dyn Logger>,
}

impl Pirs {
  pub fn new(
    exit_handler: fn(i32),
    #[cfg(not(feature = "use-default-logger"))] logger: impl Logger,
    #[cfg(feature = "use-default-logger")] log_level: LogLevel,
  ) -> Self {
    Self {
      exit_handler,
      state: State::new(commands::BUILT_IN_COMMANDS()),
      #[cfg(not(feature = "use-default-logger"))]
      logger: Box::new(logger),
      #[cfg(feature = "use-default-logger")]
      logger: Box::new(default_logger::DefaultLogger(log_level)),
    }
  }

  pub fn handle_command(&mut self, command: impl AsRef<str>) {
    let cmd = command
      .as_ref()
      .split_whitespace()
      .map(|arg| arg.trim_end().into())
      .collect::<Vec<String>>();

    let command_name = &cmd[0];

    let code = match self
      .state
      .commands
      .iter()
      .find(|command| command.names(self).contains(&&**command_name))
    {
      Some(command) => {
        self.logger.debug(&f!("executing: {}...", command_name));

        command.handle(cmd.iter().skip(1).map(|arg| &**arg).collect(), self)
      }
      None => {
        self.logger.error(&f!("command not found: {}", cmd[0]));

        1
      }
    };

    self.state.set_last_exit_code(code);
  }
}

pub trait CommandHandler: Sync + Send + std::fmt::Debug + 'static {
  fn names(&self, ctx: &Pirs) -> Vec<&str>;

  fn handle(&self, args: Vec<&str>, ctx: &Pirs) -> i32;

  fn help(&self, _ctx: &Pirs) -> &str {
    "No Help For This Command"
  }
}

pub trait Logger: Sync + std::fmt::Debug + Send + 'static {
  fn debug(&self, message: &dyn AsRef<str>);

  fn info(&self, message: &dyn AsRef<str>);

  fn warn(&self, message: &dyn AsRef<str>);

  fn error(&self, message: &dyn AsRef<str>);

  fn raw(&self, message: &dyn AsRef<str>);
}

#[cfg(feature = "use-default-logger")]
pub use default_logger::LogLevel;

#[cfg(feature = "use-default-logger")]
mod default_logger {
  use owo_colors::OwoColorize;

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
  }

  #[derive(Debug)]
  pub struct DefaultLogger(pub LogLevel);

  impl DefaultLogger {
    pub fn log_level(&self) -> u8 {
      if let Ok(level) = std::env::var("LOG_LEVEL") {
        match &*level.to_lowercase() {
          "debug" => 4,
          "info" => 3,
          "warn" => 2,
          _ => 1,
        }
      } else {
        match self.0 {
          LogLevel::Debug => 4,
          LogLevel::Info => 3,
          LogLevel::Warn => 2,
          LogLevel::Error => 1,
        }
      }
    }
  }

  impl super::Logger for DefaultLogger {
    fn debug(&self, message: &dyn AsRef<str>) {
      if self.log_level() >= 4 {
        println!("[{}]: {}", "debug".blue(), message.as_ref());
      }
    }

    fn info(&self, message: &dyn AsRef<str>) {
      if self.log_level() >= 3 {
        println!("[{}]: {}", "info".green(), message.as_ref());
      }
    }

    fn warn(&self, message: &dyn AsRef<str>) {
      if self.log_level() >= 2 {
        println!("[{}]: {}", "warn".yellow(), message.as_ref());
      }
    }

    fn error(&self, message: &dyn AsRef<str>) {
      if self.log_level() >= 1 {
        println!("[{}]: {}", "error".bright_red(), message.as_ref());
      }
    }

    fn raw(&self, message: &dyn AsRef<str>) {
      println!("{}", message.as_ref());
    }
  }
}