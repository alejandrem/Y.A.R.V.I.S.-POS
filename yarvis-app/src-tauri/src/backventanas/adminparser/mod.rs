pub mod utils;
pub mod parser_csv;
pub mod parser_excel;
pub mod parser_txt;
pub mod parser_commands;

// Re-exportar comandos de parser
pub use parser_csv::*;
pub use parser_excel::*;
pub use parser_txt::*;
