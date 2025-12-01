// Target: The Internet (Heroku) Login Test
browser.navigate("https://the-internet.herokuapp.com/login");
browser.sleep(2000);

browser.type("#username", "tomsmith");
browser.type("#password", "SuperSecretPassword!");
browser.sleep(200);

browser.click("#login button");  // More specific selector
browser.sleep(1500);
print("Login Successful!");