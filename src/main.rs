mod env;
mod handler;
mod chromedriver;

use thirtyfour::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // Initialise auth credentials
    env::init_consts();

    // Spawn main task
    let main_handle = tokio::spawn(start());
    tokio::select! {
        main_result = main_handle => {
            if let Ok(result) = main_result {
                match result {
                    Ok(_) => println!("Done..."),
                    Err(err) => eprintln!("ERROR: failed to complete task: {}", err)
                }
            }
        },
        exit = tokio::signal::ctrl_c() => match exit {
            Ok(()) => println!("Exiting..."),
            Err(err) => {
                eprintln!("ERROR: failed to shutdown: {}", err);
                eprintln!("Terminating...")
            }
        }
    }
    shutdown()
}

async fn start() -> WebDriverResult<()> {
    let server_url = chromedriver
        ::get_chromedriver_server().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    // let server_url = "http://localhost:9515".to_string();
    let mut caps = DesiredCapabilities::chrome();
    let _ = caps.set_ignore_certificate_errors();
    let _ = caps.accept_insecure_certs(true);
    let _ = caps.set_headless();
    let driver = WebDriver::new(&server_url, caps).await?;
    let start = std::time::Instant::now();
    let _ = handler::run(&driver).await.map_err(|e| {
        eprintln!("{:?}", e);
    });
    println!("Time elapsed: {:?}", start.elapsed());
    driver.quit().await?;
    Ok(())
}

fn shutdown() -> anyhow::Result<()> {
    chromedriver::stop_chromedriver();
    Ok(())
}
