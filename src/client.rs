use crate::cache;
use anyhow::{anyhow, Result};
use regex::Regex;

const AOC_URL: &str = "https://adventofcode.com";

pub struct Aocd {
    year: u16,
    day: u8,
    client: reqwest::blocking::Client,
    cache: cache::Cache,
}

impl Aocd {
    /// Create a new Aocd client.
    ///
    /// Requires a valid session cookie from adventofcode.com to be in a file named `~/.config/aocd/token`
    /// It will also require write access to `~/.cache/aocd` to cache puzzle inputs and answers.
    pub fn new(year: u16, day: u8) -> Self {
        let session_token = find_aoc_token();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(&format!("session={}", session_token)).unwrap(),
        );
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        let cache = cache::Cache::new(year, day);

        Self {
            year,
            day,
            client,
            cache,
        }
    }

    /// Get the puzzle input for the given year and day.
    ///
    /// If possible this will fetch from a local cache, and only fall back to the server if necessary.
    pub fn get_input(&self) -> String {
        if let Ok(input) = self.cache.get_input() {
            return input;
        }
        let input = self
            .client
            .get(&format!("{}/{}/day/{}/input", AOC_URL, self.year, self.day))
            .send()
            .expect("Failed to get input")
            .text()
            .expect("Failed to parse input");
        self.cache.cache_input(&input);
        input
    }

    /// Submit an answer to the given year, day, and part.
    pub fn submit(&self, part: u8, answer: impl ToString) {
        // First check if we have already cached a _correct_ answer for this puzzle.
        if let Ok(correct_answer) = self.cache.get_correct_answer(part) {
            let fill_word = if correct_answer == answer.to_string() {
                "the same"
            } else {
                "a different"
            };
            println!(
                "Part {} already solved with {} answer: {}",
                part, fill_word, correct_answer
            );
            return;
        }

        // Now check if we have already checked this particular answer before. If so we know it is wrong.
        if let Ok(response) = self.cache.get_answer_response(part, &answer.to_string()) {
            println!(
                "You've already incorrectly guessed {}, and the server responed with:\n{}",
                answer.to_string(),
                response
            );
            return;
        }

        // Only now do we actually submit the (new) answer.
        let url = format!("{}/{}/day/{}/answer", AOC_URL, self.year, self.day);
        let response = self
            .client
            .post(&url)
            .form(&[("level", part.to_string()), ("answer", answer.to_string())])
            .send()
            .expect("Faled to submit answer");

        assert!(
            response.status().is_success(),
            "Non 200 response from AoC when posting answer. Failed to submit answer. Check your token."
        );
        let response_html = response
            .text()
            .expect("Falied to read response from AoC after posting answer.");

        self.handle_answer_response(part, &answer.to_string(), &response_html);
    }

    fn handle_answer_response(&self, part: u8, answer: &str, html: &str) {
        let mut response = None;
        for line in html.lines() {
            if line.starts_with("<article>") {
                response = Some(
                    line.trim_start_matches("<article>")
                        .trim_end_matches("</article>")
                        .trim_start_matches("<p>")
                        .trim_end_matches("</p>"),
                );
            }
        }
        let response = response.expect("Failed to parse response from AoC when submitting answer.");

        let mut correct = false;
        if response.contains("That's the right answer!") {
            println!("Part {} correctly solved with answer: {}", part, answer);
            correct = true;
        } else if response.contains("That's not the right answer")
            || response.contains("You gave an answer too recently")
        {
            println!("{}", response);
        } else if response.contains("Did you already complete it") {
            // We've apparently already solved this in the past, but the cache has no memory of that.
            // In this case we look up what we've solved in the past, and cache it.
            // Then we can restart the submit flow entirely, and it should not hit this case again.
            match self.cache_past_answers() {
                Ok(()) => return self.submit(part, answer),
                _ => panic!("Failed to cache past answers, even though we thought we had solved this puzzle before. BUG!"),
            }
        }

        self.cache
            .cache_answer_response(part, answer, response, correct);
    }

    fn cache_past_answers(&self) -> Result<()> {
        println!(
            "Caching past answers for {} day {} by parsing the puzzle page.",
            self.year, self.day
        );
        let url = format!("{}/{}/day/{}/answer", AOC_URL, self.year, self.day);
        let response = self.client.get(&url).send()?.error_for_status()?;
        let response_html = response.text()?;

        println!("Here is the response from AoC:");
        println!("{}", response_html);
        let mut part1: Option<String> = None;
        let mut part2: Option<String> = None;
        let re = Regex::new(r#"Your puzzle answer was <code>(.*)</code>"#).unwrap();
        for capture in re.captures_iter(&response_html) {
            if part1.is_none() {
                part1 = Some(capture[1].to_string());
            } else {
                part2 = Some(capture[1].to_string());
            }
        }
        println!("Found past answers: {:?} {:?}", part1, part2);
        let mut found_any = false;
        if let Some(part1) = part1 {
            self.cache
                .cache_answer_response(1, &part1, "That's the right answer!", true);
            found_any = true;
        }
        if let Some(part2) = part2 {
            self.cache
                .cache_answer_response(2, &part2, "That's the right answer!", true);
            found_any = true;
        }
        if found_any {
            Ok(())
        } else {
            Err(anyhow!("Failed to find past answers"))
        }
    }
}

fn find_aoc_token() -> String {
    let path = shellexpand::tilde("~/.config/aocd/token");
    std::fs::read_to_string(path.as_ref())
        .unwrap_or_else(|_| {
            panic!(
                "{} not found. Please add this file with a valid token.",
                &path
            )
        })
        .trim()
        .to_string()
}
