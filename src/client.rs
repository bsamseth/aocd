use std::fmt::Display;

use crate::cache;
use anyhow::{anyhow, Result};
use regex::Regex;

pub struct Aocd {
    year: u16,
    day: u8,
    url: String,
    session_token: String,
    cache: cache::Cache,
}

impl Aocd {
    /// Create a new Aocd client.
    ///
    /// Requires a valid session cookie from adventofcode.com to be in a file named `~/.config/aocd/token`
    /// It will also require write access to `~/.cache/aocd` to cache puzzle inputs and answers.
    ///
    /// # Panics
    /// Panics if the session cookie is not found or the cache could not be successfully setup/initialized.
    #[must_use]
    pub fn new(year: u16, day: u8) -> Self {
        let session_token = find_aoc_token();
        let cache = cache::Cache::new(year, day, &session_token)
            .expect("Should be able to create cache for aocd");

        #[cfg(not(test))]
        let url = "https://adventofcode.com".to_string();
        #[cfg(test)]
        let url = mockito::server_url();

        Self {
            year,
            day,
            url,
            session_token,
            cache,
        }
    }

    /// Get the puzzle input for the given year and day.
    ///
    /// If possible this will fetch from a local cache, and only fall back to the server if necessary.
    ///
    /// # Panics
    /// Panics if the Advent of Code server responds with an error.
    #[must_use]
    pub fn get_input(&self) -> String {
        if let Ok(input) = self.cache.get_input() {
            return input;
        }

        let input = minreq::get(format!("{}/{}/day/{}/input", self.url, self.year, self.day))
            .with_header("Cookie", format!("session={}", self.session_token))
            .with_header("Content-Type", "text/plain")
            .send()
            .expect("Failed to get input")
            .as_str()
            .expect("Failed to parse input as string")
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string();
        self.cache
            .cache_input(&input)
            .expect("Should be able to cache input");
        input
    }

    /// Submit an answer to the given year, day, and part.
    ///
    /// # Panics
    /// Panics if the Advent of Code server responds to the submission with an error.
    pub fn submit(&self, part: u8, answer: impl Display) {
        let answer = answer.to_string();
        // First check if we have already cached a _correct_ answer for this puzzle.
        if let Ok(correct_answer) = self.cache.get_correct_answer(part) {
            if correct_answer == answer {
                println!("⭐ Part {part} already solved with the same answer: {correct_answer} ⭐");
            } else {
                println!("❌ Part {part} already solved with a different answer: {correct_answer} (you submitted: {answer}) ❌");
            }
            return;
        }

        // Now check if we have already checked this particular answer before. If so we know it is wrong.
        if let Ok(response) = self.cache.get_answer_response(part, &answer) {
            println!( "❌ You've already incorrectly guessed {answer}, and the server responed with: ❌ \n{response}");
            return;
        }

        // Only now do we actually submit the (new) answer.
        let url = format!("{}/{}/day/{}/answer", self.url, self.year, self.day);
        let formdata = format!("level={}&answer={}", part, urlencoding::encode(&answer));
        let r = minreq::post(url)
            .with_header("Cookie", format!("session={}", self.session_token))
            .with_header("Content-Type", "application/x-www-form-urlencoded")
            .with_body(formdata);
        let response = r.send().expect("Faled to submit answer");

        assert!(
            response.status_code == 200,
            "Non 200 response from AoC when posting answer. Failed to submit answer. Check your token."
        );
        let response_html = response
            .as_str()
            .expect("Falied to read response from AoC after posting answer.");

        self.handle_answer_response(part, &answer, response_html)
            .expect("Failed to handle response from AoC");
    }

    fn handle_answer_response(&self, part: u8, answer: &str, html: &str) -> Result<()> {
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

        if response.contains("That's the right answer!") {
            println!("🌟 Part {part} correctly solved with answer: {answer} 🌟");
            self.cache
                .cache_answer_response(part, answer, response, true)?;
        } else if response.contains("That's not the right answer") {
            println!("❌ {response}");
            self.cache
                .cache_answer_response(part, answer, response, false)?;
        } else if response.contains("You gave an answer too recently") {
            // Don't cache this response.
            println!("❌ {response}");
        } else if response.contains("Did you already complete it") {
            // We've apparently already solved this in the past, but the cache has no memory of that.
            // In this case we look up what we've solved in the past, and cache it.
            // Then we can restart the submit flow entirely, and it should not hit this case again.
            match self.cache_past_answers() {
                Ok(()) => self.submit(part, answer),
                _ => panic!("Failed to cache past answers, even though we thought we had solved this puzzle before. BUG!"),
            }
        }
        Ok(())
    }

    fn cache_past_answers(&self) -> Result<()> {
        println!("You appear to have answered this puzzle before, but aocd doesn't remember that.");
        println!(
            "Caching past answers for {} day {} by parsing the puzzle page.",
            self.year, self.day
        );
        let url = format!("{}/{}/day/{}/answer", self.url, self.year, self.day);
        let response = minreq::get(url)
            .with_header("Cookie", format!("session={}", self.session_token))
            .with_header("Content-Type", "text/plain")
            .send()?;
        if response.status_code != 200 {
            return Err(anyhow!(
                "Non 200 response from AoC when getting puzzle page. Failed to cache past answers. Check your token."
            ));
        }
        let response_html = response.as_str()?;

        let mut part1: Option<String> = None;
        let mut part2: Option<String> = None;
        let re = Regex::new(r#"Your puzzle answer was <code>(.*?)</code>"#).unwrap();
        for capture in re.captures_iter(response_html) {
            if part1.is_none() {
                part1 = Some(capture[1].to_string());
            } else {
                part2 = Some(capture[1].to_string());
            }
        }
        println!("Found past answers: {part1:?} {part2:?}");
        let mut found_any = false;
        if let Some(part1) = part1 {
            self.cache
                .cache_answer_response(1, &part1, "That's the right answer!", true)?;
            found_any = true;
        }
        if let Some(part2) = part2 {
            self.cache
                .cache_answer_response(2, &part2, "That's the right answer!", true)?;
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
    if let Ok(session) = std::env::var("AOC_SESSION").or_else(|_| std::env::var("AOC_TOKEN")) {
        return session.trim().to_string();
    }

    let token_path = std::env::var("AOC_TOKEN_PATH")
        .unwrap_or_else(|_| shellexpand::tilde("~/.config/aocd/token").to_string());
    std::fs::read_to_string(token_path)
        .unwrap_or_else(|_| {
            panic!(
                "No AoC session token found. See https://crates.io/crates/aocd for how to set it.",
            )
        })
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    struct TestClientBuilder {
        year: u16,
        day: u8,
        input: Option<String>,
    }

    impl TestClientBuilder {
        fn new() -> Self {
            TestClientBuilder {
                year: 2015,
                day: 1,
                input: None,
            }
        }
        fn year(mut self, year: u16) -> Self {
            self.year = year;
            self
        }
        fn day(mut self, day: u8) -> Self {
            self.day = day;
            self
        }
        fn input(mut self, input: &str) -> Self {
            self.input = Some(input.to_string());
            self
        }
        fn run<F, T>(&self, test: F) -> Result<T>
        where
            T: std::panic::RefUnwindSafe,
            F: FnOnce(&Aocd) -> Result<T>
                + std::panic::UnwindSafe
                + std::panic::RefUnwindSafe
                + Copy,
        {
            let cache_path = std::env::temp_dir().join("aocd-tests");
            let _ignore = std::fs::remove_dir_all(&cache_path);

            temp_env::with_vars(
                vec![
                    ("AOC_SESSION", Some("test-session")),
                    ("AOC_CACHE_DIR", Some(cache_path.to_str().unwrap())),
                ],
                move || {
                    let client = Aocd::new(self.year, self.day);
                    if let Some(input) = &self.input {
                        let url = format!("/{}/day/{}/input", client.year, client.day);
                        let m = mock("GET", url.as_str())
                            .with_status(200)
                            .with_header("Content-Type", "text/plain")
                            .with_body(input)
                            .expect(1)
                            .create();
                        let result = test(&client);
                        m.assert();
                        result
                    } else {
                        test(&client)
                    }
                },
            )
        }
    }

    #[test]
    fn test_new_client() -> Result<()> {
        TestClientBuilder::new().year(2022).day(1).run(|client| {
            assert_eq!(client.year, 2022);
            assert_eq!(client.day, 1);
            assert_eq!(client.url, mockito::server_url());
            Ok(())
        })
    }

    #[test]
    fn test_get_input() -> Result<()> {
        TestClientBuilder::new()
            .year(2022)
            .day(1)
            .input("test input")
            .run(|client| {
                assert_eq!(client.get_input(), "test input");
                // A second call will trigger a cache hit. If it doesn't the test will fail because
                // the mock endpoint only expects a single call.
                assert_eq!(client.get_input(), "test input");
                Ok(())
            })
    }

    #[test]
    #[ignore]
    fn test_submit_answer() {
        todo!()
    }

    #[test]
    fn test_find_aoc_token_env() {
        temp_env::with_var("AOC_SESSION", Some("testsession"), || {
            assert_eq!(find_aoc_token(), "testsession");
        });
        temp_env::with_var("AOC_TOKEN", Some("testtoken"), || {
            assert_eq!(find_aoc_token(), "testtoken");
        });
    }

    #[test]
    fn test_find_aoc_token_file() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("aocd-token");
        let mut file = File::create(&file_path)?;
        writeln!(file, "testtokenintempfile")?;

        temp_env::with_var("AOC_TOKEN_PATH", Some(&file_path), || {
            assert_eq!(find_aoc_token(), "testtokenintempfile");
            Ok(())
        })
    }
}
