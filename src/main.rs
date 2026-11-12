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

    do_exercise(&driver, &database, &translator).await?;

    driver.quit().await?;

    Ok(())
}

async fn do_session(driver: &WebDriver, database: &Tree, translator: &Translator) -> Result<(), Box<dyn Error>> {
    println!("Doing session");
    click(driver, By::ClassName("btn-start-session")).await?;
    click(driver, By::Id("continue_session_button")).await?;

    while element_exists(driver, By::Id("question_usage")) {
        solve_question(driver, database, translator).await?
    }

    Ok(())
}

async fn solve_question(driver: &WebDriver, database: &Tree, translator: &Translator) -> Result<(), Box<dyn Error>> {
    let sentence = get_text(driver, By::Id("question_usage")).await?;
    let hint = get_text(driver, By::ClassName("word_data")).await?;

    print!("Question: {hint}");

    let key = (sentence + &*hint).into_bytes();
    
    let answer = match database.begin()?.get(&key)? {
        Some(val) => {
            String::from_utf8(val)?
        }
        None => {
            let mut word = hint;
            word.truncate(word.find(','));
            translator.translate_sync(word, "pl", "en")?
        }
    };
    
    println!("Guessed answer: {answer}");

    send_keys(driver, By::Id("answer"), &answer).await?;
    click(driver, By::Id("check")).await?;

    let correct_answer = get_text(driver, By::Id("word")).await?;

    println!("Correct answer: {correct_answer}");

    if answer != correct_answer {
        let mut tx = database.begin()?;
        tx.set(&key, correct_answer.into_bytes())?;
        tx.commit().await?;
    }

    click(driver, By::Id("nextword")).await?;

    Ok(())
}

// async fn print_page(driver: &WebDriver) -> Result<(), WebDriverError> {
//     println!("{}", driver.source().await?);
//     Ok(())
// }

async fn login(driver: &WebDriver) -> Result<(), WebDriverError> {
    driver.goto("https://instaling.pl/teacher.php?page=login").await?;

    send_keys(driver, By::Id("log_email"), &read_secret("username")?).await?;
    send_keys(driver, By::Id("log_password"), &read_secret("password")?).await?;
    click(driver, By::Css("button[type=submit]")).await?;
    
    Ok(())
}

async fn element_exists(driver: &WebDriver, selector: By) -> Result<Bool, WebDriverError> {
    driver
        .query(selector)
        .exists()
        .await?
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
