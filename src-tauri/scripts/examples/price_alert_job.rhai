// Target: Specific Book Page
browser.navigate("https://books.toscrape.com/catalogue/tipping-the-velvet_999/index.html");

let price_str = browser.extract_text(".price_color"); // e.g., "£53.74"
// Remove currency symbol and parse
let price = parse_float(price_str.sub_string(1));

if price < 55.00 {
    print(`ALERT: Price dropped to ${price}!`);
    // In a real app, this might trigger a webhook
    // http.post("https://api.my-slack-bot.com/alert", #{ item: "Velvet", price: price });
} else {
    print(`Price is still high: ${price}`);
}
