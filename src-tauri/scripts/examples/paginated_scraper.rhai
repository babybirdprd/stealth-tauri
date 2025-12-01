// Target: Books to Scrape
browser.navigate("https://books.toscrape.com/");
let books = [];

loop {
    let titles = browser.extract_all("h3 > a", "title");
    books += titles;
    print(`Collected ${books.len()} books so far...`);

    if browser.exists(".next > a") {
        browser.click(".next > a");
        sleep(1000); // Respectful crawl delay
    } else {
        break;
    }

    if books.len() >= 40 { break; } // Limit for demo
}

save_json("books_dump.json", books);
