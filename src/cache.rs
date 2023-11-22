use anyhow::Result;

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
        std::fs::write(format!("{prefix}-resp-{answer}"), response)?;
        if correct {
            std::fs::write(format!("{prefix}-correct"), answer)?;
        }

        Ok(())
    }

    pub fn get_correct_answer(&self, part: u8) -> Result<String> {
        let prefix = self.answer_cache_file_prefix(part);
        let answer = std::fs::read_to_string(format!("{prefix}-correct"))?;
        Ok(answer)
    }

    pub fn get_answer_response(&self, part: u8, answer: &str) -> Result<String> {
        let prefix = self.answer_cache_file_prefix(part);
        let response = std::fs::read_to_string(format!("{prefix}-resp-{answer}"))?;
        Ok(response)
    }

    pub fn get_input(&self) -> Result<String> {
        let input = std::fs::read_to_string(self.input_cache_file())?;
        Ok(input)
    }

    pub fn cache_input(&self, input: &str) -> Result<()> {
        std::fs::write(self.input_cache_file(), input)?;
        Ok(())
    }
}
