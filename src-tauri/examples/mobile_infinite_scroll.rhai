// Target: Infinite Scroll Sandbox
browser.navigate("https://quotes.toscrape.com/scroll");
browser.wait_for_element(".quote");

for i in 0..5 {
    print(`Scrolling page... iteration ${i}`);
    // Simulate a "Thumb Swipe" gesture (smooth scroll)
    browser.scroll_smooth(0, 500);
    // Wait for network requests to settle
    sleep(1500);
}
print("Finished scrolling.");
