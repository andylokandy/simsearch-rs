use std::fs::File;
use std::io::Read as _;

use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};
use inquire::{
    autocompletion::{Autocomplete, Replacement},
    CustomUserError, Text,
};

use simsearch::SimSearch;

fn main() {
    inquire::set_global_render_config(get_render_config());

    Text::new("Search for a book:")
        .with_autocomplete(BookSearcher::load())
        .with_help_message("Try typing 'old man'")
        .with_page_size(15)
        .prompt()
        .unwrap();
}

#[derive(Clone)]
pub struct BookSearcher {
    engine: SimSearch<String>,
}

impl BookSearcher {
    pub fn load() -> Self {
        let mut engine = SimSearch::new();

        let mut file = File::open("./books.json").unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let j = json::parse(&content).unwrap();
        for title in j.members() {
            engine.insert(title.as_str().unwrap().to_string(), title.as_str().unwrap());
        }

        BookSearcher { engine }
    }
}

impl Autocomplete for BookSearcher {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        Ok(self.engine.search(input))
    }

    fn get_completion(
        &mut self,
        _: &str,
        _: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        Ok(None)
    }
}

fn get_render_config() -> RenderConfig {
    let mut render_config = RenderConfig::default();

    render_config.prompt_prefix = Styled::new(">").with_fg(Color::LightRed);
    render_config.highlighted_option_prefix = Styled::new("*").with_fg(Color::LightYellow);
    render_config.option = StyleSheet::new().with_fg(Color::DarkBlue);
    render_config.help_message = StyleSheet::new().with_fg(Color::LightYellow);

    render_config
}
