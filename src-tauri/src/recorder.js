(function() {
    if (window.__PHANTOM_RECORDER_ACTIVE__) return;
    window.__PHANTOM_RECORDER_ACTIVE__ = true;

    let hoveredElement = null;

    function getSelector(el) {
        if (!el) return "";
        // 1. ID
        if (el.id) return '#' + CSS.escape(el.id);

        // 2. Attributes
        if (el.hasAttribute('data-testid')) return `[data-testid="${CSS.escape(el.getAttribute('data-testid'))}"]`;
        if (el.hasAttribute('aria-label')) return `[aria-label="${CSS.escape(el.getAttribute('aria-label'))}"]`;

        // 3. Class
        if (el.className && typeof el.className === 'string') {
            const classes = el.className.split(/\s+/).filter(c => c);
            if (classes.length > 0) {
                 const classSel = '.' + classes.map(c => CSS.escape(c)).join('.');
                 // Check uniqueness
                 if (document.querySelectorAll(classSel).length === 1) return classSel;
            }
        }

        // 4. Path
        let path = [];
        let current = el;
        while (current && current.nodeType === Node.ELEMENT_NODE) {
            let selector = current.nodeName.toLowerCase();
            if (current.id) {
                selector += '#' + CSS.escape(current.id);
                path.unshift(selector);
                break; // ID is usually unique enough, stop here
            } else {
                let sib = current, nth = 1;
                while (sib = sib.previousElementSibling) {
                    if (sib.nodeName.toLowerCase() === selector) nth++;
                }
                if (nth !== 1) selector += ":nth-of-type("+nth+")";
            }
            path.unshift(selector);
            current = current.parentNode;
        }
        return path.join(" > ");
    }

    // Styles for outline
    const style = document.createElement('style');
    style.innerHTML = `
        .phantom-recording-highlight {
            outline: 2px solid red !important;
            cursor: crosshair !important;
        }
    `;
    document.head.appendChild(style);

    document.addEventListener('mouseover', (e) => {
        if (hoveredElement) {
            hoveredElement.classList.remove('phantom-recording-highlight');
        }
        hoveredElement = e.target;
        hoveredElement.classList.add('phantom-recording-highlight');
    }, true);

    document.addEventListener('mouseout', (e) => {
        if (hoveredElement) {
             hoveredElement.classList.remove('phantom-recording-highlight');
             hoveredElement = null;
        }
    }, true);

    document.addEventListener('click', (e) => {
        if (!e.isTrusted) return; // Ignore synthetic clicks

        e.preventDefault();
        e.stopPropagation();

        const selector = getSelector(e.target);

        window.__TAURI__.core.invoke('recorder_event', {
            event_type: "click",
            selector: selector
        }).then(() => {
            // Re-trigger click
            e.target.click();
        });

    }, true);

    // Capture input changes
    document.addEventListener('change', (e) => {
         if (!e.isTrusted) return;
         const selector = getSelector(e.target);
         window.__TAURI__.core.invoke('recorder_event', {
             event_type: "type",
             selector: selector,
             value: e.target.value
         });
    }, true);

    console.log("Phantom Recorder Activated");
})();
