use anyhow::Result;
use std::{
    fs::File,
    io::{Read, Write},
};

pub struct Cache {
    year: u16,
    day: u8,
    cache_directory: String,
}

impl Cache {
    pub fn new(year: u16, day: u8, session: &str) -> Result<Self> {
        let directory = std::env::var("AOC_CACHE_DIR")
            .or_else(|_| std::env::var("XDG_CACHE_HOME"))
            .unwrap_or_else(|_| shellexpand::tilde("~/.cache/aocd").to_string());
        let directory = format!("{directory}/{session}");

        let inputs_directory = format!("{directory}/inputs");
        let answers_directory = format!("{directory}/answers");

        std::fs::create_dir_all(inputs_directory)?;
        std::fs::create_dir_all(answers_directory)?;

        Ok(Self {
            year,
            day,
            cache_directory: directory,
        })
    }

    fn answer_cache_file_prefix(&self, part: u8) -> String {
        format!(
            "{directory}/answers/{year}-{day:02}-{part}",
            directory = self.cache_directory,
            year = self.year,
            day = self.day,
            part = part
        )
    }

    fn input_cache_file(&self) -> String {
        format!(
            "{directory}/inputs/{year}-{day:02}",
            directory = self.cache_directory,
            year = self.year,
            day = self.day
        )
    }

    pub fn cache_answer_response(
        &self,
        part: u8,
        answer: &str,
        response: &str,
        correct: bool,
    ) -> Result<()> {
        let prefix = self.answer_cache_file_prefix(part);
        File::create(format!("{prefix}-resp-{answer}"))?.write_all(response.as_bytes())?;
        if correct {
            File::create(format!("{prefix}-correct"))?.write_all(answer.as_bytes())?;
        }

        Ok(())
    }

    pub fn get_correct_answer(&self, part: u8) -> Result<String> {
        let prefix = self.answer_cache_file_prefix(part);
        let mut file = File::open(format!("{prefix}-correct"))?;
        let mut answer = String::new();
        file.read_to_string(&mut answer)?;
        Ok(answer)
    }

    pub fn get_answer_response(&self, part: u8, answer: &str) -> Result<String> {
        let prefix = self.answer_cache_file_prefix(part);
        let mut file = File::open(format!("{prefix}-resp-{answer}"))?;
        let mut response = String::new();
        file.read_to_string(&mut response)?;
        Ok(response)
    }

    pub fn get_input(&self) -> Result<String> {
        let mut file = File::open(self.input_cache_file())?;
        let mut input = String::new();
        file.read_to_string(&mut input)?;
        Ok(input)
    }

    pub fn cache_input(&self, input: &str) -> Result<()> {
        let mut file = File::create(self.input_cache_file())?;
        file.write_all(input.as_bytes())?;
        Ok(())
    }
}
