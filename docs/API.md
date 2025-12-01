# Phantom Browser Scripting API

The Phantom Browser uses Rhai, a lightweight embedded scripting language for Rust. The global `browser` object provides control over the webview.

## Browser API

### `browser.navigate(url: string)`

Navigates the webview to the specified URL.

```rust
browser.navigate("https://example.com");
```

### `browser.click(selector: string)`

Clicks the first DOM element matching the given CSS selector.

```rust
browser.click("#login-button");
```

### `browser.wait_for_selector(selector: string)`

Blocks execution until the element matching the selector appears in the DOM. Useful for waiting for page loads or dynamic content.

```rust
browser.wait_for_selector(".dashboard");
```

### `browser.extract_text(selector: string) -> string`

Extracts and returns the `innerText` of the first element matching the selector. Returns an empty string if not found.

```rust
let title = browser.extract_text("h1");
print(title);
```

## Standard Rhai Functions

You can use standard Rhai features like variables, loops, and control flow.

```rust
let items = ["#a", "#b", "#c"];
for item in items {
    if browser.extract_text(item) == "Target" {
        browser.click(item);
        break;
    }
}
```
