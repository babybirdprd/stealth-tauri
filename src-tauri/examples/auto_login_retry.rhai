let attempts = 0;
let max_retries = 3;

loop {
    if attempts >= max_retries { throw "Max login attempts reached"; }

    browser.navigate("https://the-internet.herokuapp.com/login");
    browser.type_text("#username", "wrong_user"); // Intentional fail
    browser.type_text("#password", "wrong_pass");
    browser.click("button[type='submit']");

    if browser.exists(".flash.error") {
        print(`Login failed (Attempt ${attempts + 1}). Retrying...`);
        attempts += 1;
        sleep(1000);
    } else {
        break; // Success
    }
}
