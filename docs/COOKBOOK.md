# Phantom Browser Cookbook

**Note:** When running scripts, the Phantom Browser opens a separate "Target" window to perform the automation. This ensures your script logic and the browser session are isolated from the Command Center.

## 1. Login with 2FA (Waiting for User Input)

This script navigates to a login page and waits for the user to manually handle the 2FA step.

```rust
print("Navigating to login...");
browser.navigate("https://github.com/login");

browser.wait_for_selector("#login_field");
print("Please log in manually.");

// Wait for a specific element that appears only after login
browser.wait_for_selector(".AppHeader-context-item");
print("Login successful! Proceeding with automation...");
```

## 2. Infinite Scroll Scraping

Demonstrates handling pagination or scrolling by clicking "More" buttons.

```rust
browser.navigate("https://news.ycombinator.com");

for i in 0..3 {
    print("Scraping Page " + (i + 1));

    // Extract first headline
    let title = browser.extract_text(".titleline > a");
    print("Top Story: " + title);

    // Go to next page
    if browser.extract_text(".morelink") != "" {
        browser.click(".morelink");
        browser.wait_for_selector("#hnmain"); // Wait for reload
    } else {
        break;
    }
}
```

## 3. Data Extraction

Extracts data from specific elements.

```rust
browser.navigate("https://example.com");
browser.wait_for_selector("h1");

let title = browser.extract_text("h1");
let desc = browser.extract_text("p");

print("Title: " + title);
print("Description: " + desc);
```

## 4. Proxy Rotation (Concept)

Demonstrates how logic would loop through proxies (Network layer coming in Phase 2).

```rust
let proxies = ["192.168.1.1", "192.168.1.2"];

for proxy in proxies {
    print("Rotating to " + proxy);
    // set_proxy(proxy);

    browser.navigate("https://ifconfig.me");
    let ip = browser.extract_text("body");
    print("Visible IP: " + ip);
}
```

## 5. Mobile Emulation

Navigate to a mobile-optimized site. *Ensure "Mobile iPhone" profile is selected in the UI.*

```rust
browser.navigate("https://m.wikipedia.org");
browser.wait_for_selector(".header-container");

print("Loaded Mobile Wikipedia");
let feature = browser.extract_text("#featured-article-display a");
print("Featured Article: " + feature);
```
