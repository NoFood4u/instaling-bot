use thirtyfour::prelude::*;
use thirtyfour::manager::*;
use surrealkv::{Tree, TreeBuilder};
use translators::{GoogleTranslator, Translator};
use std::fs::File;
use std::io::Read;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut caps = DesiredCapabilities::firefox();
    // caps.add_arg("--headless")?;
    let driver = WebDriver::managed(caps)
        .driver_binary(BrowserKind::Firefox, "geckodriver")
        // .stdio(StdioMode::Inherit)
        .offline()
        .await?;

    let database = TreeBuilder::new()
        .with_path("database".into())
        .build()?;

    let translator = GoogleTranslator::default();

    login(&driver).await?;

    do_session(&driver, &database, &translator).await?;

    driver.quit().await?;

    Ok(())
}

async fn do_session(driver: &WebDriver, database: &Tree, translator: &GoogleTranslator) -> Result<(), Box<dyn Error>> {
    println!("Doing session");
    click(driver, By::ClassName("btn-start-session")).await?;
    driver
        .query(By::Id("start_session_button"))
        .and_displayed()
        .or(By::Id("continue_session_button"))
        .and_displayed()
        .single()
        .await?
        .click()
        .await?;

    let (mut correct_count, mut total_count) = (0, 0);
    while element_visible(driver, By::Id("question_usage")).await? {
        let is_correct = solve_question(driver, database, translator).await?;
        if is_correct {
            correct_count += 1;
        }
        total_count += 1;
    }

    println!("Finished session with success rate: {}%", 100 * correct_count / total_count);

    Ok(())
}

async fn guess_answer(database: &Tree, translator: &GoogleTranslator, key: &[u8], hint: String) -> Result<(String, bool), Box<dyn Error>> {
    if let Some(answer) = database.begin()?.get(key)? {
        return Ok((String::from_utf8(answer)?, false));
    }

    let mut word = hint;
    if let Some(size) = word.find(';') {
        word.truncate(size);
    }
    else if let Some(size) = word.find(',') {
        word.truncate(size);
    }
    Ok((translator.translate_async(&word, "pl", "en").await?, true))
}

async fn solve_question(driver: &WebDriver, database: &Tree, translator: &GoogleTranslator) -> Result<bool, Box<dyn Error>> {
    let sentence = get_text(driver, By::Id("question_usage")).await?;
    let hint = get_text(driver, By::ClassName("word_data")).await?;

    println!("Question: {} -- {}", sentence.replace("\n", ""), hint.replace("\n", ""));

    let key = (sentence + &*hint).into_bytes();
    let (answer, should_update_database) = guess_answer(database, translator, &key, hint).await?;
    
    if should_update_database {
        println!("Guessed answer: {answer}");
    } else {
        println!("Remembered answer: {answer}");
    }

    send_keys(driver, By::Id("answer"), &answer).await?;
    click(driver, By::Id("check")).await?;

    let answer_result = get_text(driver, By::Id("answer_result")).await?;
    let is_answer_incorrect = answer_result == "Niepoprawnie";

    if answer_result == "Dobrze!" {
        if should_update_database {
            let correct_answer = get_text(driver, By::Id("word")).await?;
            learn_word(database, &key, correct_answer).await?;
        }
    } else if is_answer_incorrect {
        let correct_answer = get_text(driver, By::Id("word")).await?;
        learn_word(database, &key, correct_answer).await?;
    } else {
        forget_word(database, &key).await?;
    }

    if is_answer_incorrect {
        click(driver, By::Id("nextword")).await?;
        return Ok(false);
    }

    click(driver, By::Id("nextword")).await?;
    Ok(true)
}

async fn learn_word(database: &Tree, key: &[u8], word: String) -> Result<(), Box<dyn Error>> {
    println!("Learned new word: {word}");

    let mut tx = database.begin()?;
    tx.set(key, word.into_bytes())?;
    tx.commit().await?;

    Ok(())
}

async fn forget_word(database: &Tree, key: &[u8]) -> Result<(), Box<dyn Error>> {
    println!("Reset this answer");

    let mut tx = database.begin()?;
    tx.set(key, b"")?;
    tx.commit().await?;

    Ok(())
}

async fn login(driver: &WebDriver) -> Result<(), WebDriverError> {
    driver.goto("https://instaling.pl/teacher.php?page=login").await?;

    send_keys(driver, By::Id("log_email"), &read_secret("username")?).await?;
    send_keys(driver, By::Id("log_password"), &read_secret("password")?).await?;
    click(driver, By::Css("button[type=submit]")).await?;
    
    Ok(())
}

async fn element_visible(driver: &WebDriver, selector: By) -> Result<bool, WebDriverError> {
    driver
        .query(selector)
        .and_displayed()
        .exists()
        .await
}

async fn get_text(driver: &WebDriver, selector: By) -> Result<String, WebDriverError> {
    let element = driver
        .query(selector)
        .single()
        .await?;

    element
        .wait_until()
        .condition(|elem: WebElement| async move {
            let text = elem.text().await?;
            Ok(!text.is_empty())
        })
        .await?;

    element.text().await
}

async fn send_keys(driver: &WebDriver, selector: By, keys: &String) -> Result<(), WebDriverError> {
    driver
        .query(selector)
        .single()
        .await?
        .send_keys(keys)
        .await
}

async fn click(driver: &WebDriver, selector: By) -> Result<(), WebDriverError> {
    driver
        .query(selector)
        .single()
        .await?
        .click()
        .await
}

fn read_secret(name: &str) -> std::io::Result<String> {
    let mut file = File::open(format!("secrets/{}", name))?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
}
