/**
 * Bootstrap Loader for Toon Dash
 */

(function() {
    'use strict';

    const WASM_MODULE = 'toon_dash.js';
    const MEMORY_INITIAL = 64; // MB
    const MEMORY_MAXIMUM = 256; // MB

    const elements = {
        splash: document.getElementById('splash'),
        progress: document.getElementById('splash-progress'),
        status: document.getElementById('splash-status'),
        canvas: document.getElementById('game-canvas'),
        debug: document.getElementById('debug-overlay')
    };

    function updateProgress(percent, text) {
        elements.progress.style.width = percent + '%';
        elements.status.textContent = text;
    }

    async function loadScript(src) {
        return new Promise((resolve, reject) => {
            const script = document.createElement('script');
            script.src = src;
            script.onload = resolve;
            script.onerror = () => reject(new Error('Failed: ' + src));
            document.head.appendChild(script);
        });
    }

    async function loadWasm() {
        updateProgress(10, 'Loading engine...');

        try {
            await loadScript(WASM_MODULE);
            updateProgress(50, 'Initializing...');

            if (typeof wasm_bindgen !== 'undefined') {
                await wasm_bindgen({ canvas: elements.canvas });
            }

            updateProgress(100, 'Ready!');
            setTimeout(hideSplash, 300);
            return true;

        } catch (error) {
            console.error('[Bootstrap] Error:', error);
            updateProgress(0, 'Error: ' + error.message);
            return false;
        }
    }

    function hideSplash() {
        elements.splash.classList.add('hidden');
    }

    function setupInput() {
        document.addEventListener('keydown', (e) => {
            const key = PAL.mapKey(e.keyCode);
            if (key && ['Up', 'Down', 'Left', 'Right', 'Back'].includes(key)) {
                e.preventDefault();
            }
        }, true);

        document.addEventListener('contextmenu', (e) => e.preventDefault());
    }

    function setupLifecycle() {
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                console.log('[Lifecycle] Hidden');
            } else {
                console.log('[Lifecycle] Visible');
            }
        });
    }

    function checkDebug() {
        const params = new URLSearchParams(window.location.search);
        if (params.has('debug')) {
            elements.debug.classList.add('visible');
        }
    }

    async function init() {
        console.log('[Bootstrap] Toon Dash starting...');

        await new Promise(r => requestAnimationFrame(r));
        await new Promise(r => requestAnimationFrame(r));

        updateProgress(5, 'Initializing...');
        setupInput();
        setupLifecycle();
        checkDebug();

        await loadWasm();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();