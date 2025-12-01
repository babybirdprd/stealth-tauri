pub fn generate_injection_script(seed: u64) -> String {
    let mut rng = Lcg::new(seed);

    // Generate noise for Canvas
    let r_noise = rng.range(-2.0, 2.0).round();
    let g_noise = rng.range(-2.0, 2.0).round();
    let b_noise = rng.range(-2.0, 2.0).round();
    let a_noise = rng.range(-2.0, 2.0).round();

    // Generate noise for Audio
    let audio_noise = rng.range(-0.0001, 0.0001);

    // Generate WebGL Vendor/Renderer
    // Simple list for demonstration. In a real scenario, this would be more comprehensive.
    let vendors = vec!["Google Inc.", "Intel Inc.", "NVIDIA Corporation"];
    let renderers = vec![
        "ANGLE (Intel, Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        "ANGLE (NVIDIA, NVIDIA GeForce GTX 1050 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)",
        "ANGLE (AMD, AMD Radeon RX 580 Direct3D11 vs_5_0 ps_5_0, D3D11)"
    ];
    let vendor = vendors[rng.next() as usize % vendors.len()];
    let renderer = renderers[rng.next() as usize % renderers.len()];

    format!(r#"
    (function() {{
        // --- Canvas Spoofing ---
        const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
        const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData;

        const noise = {{ r: {}, g: {}, b: {}, a: {} }};

        CanvasRenderingContext2D.prototype.getImageData = function(x, y, w, h) {{
            const imageData = originalGetImageData.apply(this, arguments);
            for (let i = 0; i < imageData.data.length; i += 4) {{
                imageData.data[i] = Math.max(0, Math.min(255, imageData.data[i] + noise.r));
                imageData.data[i+1] = Math.max(0, Math.min(255, imageData.data[i+1] + noise.g));
                imageData.data[i+2] = Math.max(0, Math.min(255, imageData.data[i+2] + noise.b));
                imageData.data[i+3] = Math.max(0, Math.min(255, imageData.data[i+3] + noise.a));
            }}
            return imageData;
        }};

        HTMLCanvasElement.prototype.toDataURL = function(type, encoderOptions) {{
            // Force a re-render or manipulation if needed, but for simple noise
            // checking usually involves drawing to a canvas and reading it back.
            // Since we hooked getImageData, if the user code calls getImageData it gets noise.
            // If they use toDataURL, we might need to actually draw to a temp canvas, apply noise, and return that.
            // For this implementation, we'll hook the context operations primarily.
            // A more robust implementation handles toDataURL by rendering the current canvas to an offscreen one, applying noise, and exporting.
            // But 'Consistent Noise' often just means modifying the pixel readback.

            // Note: properly hooking toDataURL is complex because we can't easily "read" the pixels without getting tainted canvas errors or recursion.
            // A common simplified approach is to override fillText/strokeText to slightly alter positions,
            // effectively changing the hash without post-processing pixels.
            // For now, we will leave toDataURL as is, assuming the bot detection relies on getImageData or that
            // we accept slight risk here for the Phase 2 MVP.
            return originalToDataURL.apply(this, arguments);
        }};

        // --- Audio Spoofing ---
        const originalGetFloatFrequencyData = AnalyserNode.prototype.getFloatFrequencyData;
        const audioNoise = {};

        AnalyserNode.prototype.getFloatFrequencyData = function(array) {{
            const ret = originalGetFloatFrequencyData.apply(this, arguments);
            for (let i = 0; i < array.length; i++) {{
                array[i] += audioNoise;
            }}
            return ret;
        }};

        // --- WebGL Spoofing ---
        const getParameter = WebGLRenderingContext.prototype.getParameter;
        WebGLRenderingContext.prototype.getParameter = function(parameter) {{
            // 37445 = UNMASKED_VENDOR_WEBGL
            // 37446 = UNMASKED_RENDERER_WEBGL
            if (parameter === 37445) return "{}";
            if (parameter === 37446) return "{}";
            return getParameter.apply(this, arguments);
        }};

        const getParameter2 = WebGL2RenderingContext.prototype.getParameter;
        WebGL2RenderingContext.prototype.getParameter = function(parameter) {{
            if (parameter === 37445) return "{}";
            if (parameter === 37446) return "{}";
            return getParameter2.apply(this, arguments);
        }};

        // --- Rects/Resolution Spoofing (Subpixel) ---
        // Adding tiny noise to getBoundingClientRect
        const originalGetBCR = Element.prototype.getBoundingClientRect;
        Element.prototype.getBoundingClientRect = function() {{
            const rect = originalGetBCR.apply(this, arguments);
            // We can't modify the DOMRectReadOnly directly easily, so we proxy it.
            // But for high-perf, we'll just return the original for now or wrap it if strictly needed.
            // PRD mentions it, let's try a light wrapper.
            return {{
                x: rect.x,
                y: rect.y,
                width: rect.width + {},
                height: rect.height + {},
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
                left: rect.left
            }};
        }};

        console.log("Phantom Browser: Digital Mask v2 Applied");
    }})();
    "#,
    r_noise, g_noise, b_noise, a_noise,
    audio_noise,
    vendor, renderer,
    vendor, renderer,
    rng.range(-0.01, 0.01), rng.range(-0.01, 0.01) // Rect noise
    )
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_float(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_float()
    }
}
