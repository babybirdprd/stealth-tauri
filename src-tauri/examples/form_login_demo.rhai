// Target: The Internet (Heroku) Login Test
browser.navigate("https://the-internet.herokuapp.com/login");

browser.type_text("#username", "tomsmith");
browser.type_text("#password", "SuperSecretPassword!");
browser.click("button[type='submit']");

browser.wait_for_element(".flash.success");
print("Login Successful!");
